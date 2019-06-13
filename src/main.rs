#[macro_use]
extern crate clap;
extern crate semver;
extern crate toml_edit;

use std::convert::TryFrom;
use std::fs;
use std::io::Write;
use std::ops::Deref;

use clap::{App, AppSettings, Arg, ArgGroup, ArgMatches, SubCommand};
use semver::{Identifier, Version};
use toml_edit::{value, Document};

/// semver::Version does not implement converting
/// its version metadata labels (pre-release and build information)
/// into string, so in order to make rendering those labels
/// consistent that's implemented here via a newtype

/// Newtype that wraps semver::Version's pre and build
/// properties in order to allow implementing string conversion
/// on them
struct VersionMetadata(Vec<Identifier>);

/// String conversion for semver::Version's pre and build
/// properties.
impl From<VersionMetadata> for String {
    /// The semver spec states that pre-release and build
    /// information consists of a sequence of alphanumeric
    /// identifiers joined by the `.` character.
    fn from(meta: VersionMetadata) -> String {
        meta.iter()
            .map(|item| item.to_string())
            .collect::<Vec<String>>()
            .join(".")
    }
}

/// Conversion from a semver metadata label to a Vec<semver::Identifier> for use
/// in the properties of semver::Version.
impl TryFrom<&str> for VersionMetadata {
    type Error = &'static str;

    /// The semver spec states that pre-release and build labels have the same
    /// spec, except for the way they are joined to the main version - pre-release
    /// is joined by a `-` and build by `+`. This is used to get around the fact that
    /// semver does not currently (2019-06-12) provide a way to parse just a metadata
    /// label - the label is formatted into some junk version, into the pre-release position,
    /// the whole thing is parsed, and finally the label itself is returned.
    fn try_from(meta: &str) -> Result<VersionMetadata, Self::Error> {
        Ok(VersionMetadata(
            Version::parse(&format!("0.0.0-{}", meta))
                .unwrap()
                .pre,
        ))
    }
}

impl Deref for VersionMetadata {
    type Target = Vec<Identifier>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn read_manifest(path: &str) -> Document {
    fs::read_to_string(path)
        .expect("Could not find Cargo.toml")
        .parse::<Document>()
        .expect("Invalid Cargo.toml")
}

/// Reads the package version string of the given manifest document
/// and parses it into a semver::Version.
fn read_version(manifest: &Document) -> Version {
    // Since we expect the Cargo.toml manifest of an actual Rust crate
    // it is safe to assume that:
    // 1. there is a package section with a version member
    let version_str = manifest["package"]["version"].as_str().unwrap();

    // and
    // 2. the version string is in a valid semver format.
    Version::parse(version_str).expect(&format!(
        "Invalid package version: {} in Cargo.toml",
        version_str
    ))
}

/// Reads the version component chosen from the command line and
/// prints it to screen.
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
        String::from(VersionMetadata(version.pre))
    } else if matches.is_present("build") {
        String::from(VersionMetadata(version.build))
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
        version.pre = VersionMetadata::try_from(pre).unwrap().0;
    } else if let Some(build) = matches.value_of("build") {
        version.build = VersionMetadata::try_from(build).unwrap().0;
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
        .settings(&[AppSettings::SubcommandRequiredElseHelp])
        .subcommand(
            SubCommand::with_name("read")
                .arg(
                    Arg::with_name("version")
                        .long("version")
                        .help("Print the VERSION set in the given manifest."),
                )
                .arg(
                    Arg::with_name("major")
                        .long("major")
                        .help("Print the MAJOR version of this package."),
                )
                .arg(
                    Arg::with_name("minor")
                        .long("minor")
                        .help("Print the MINOR version of this package."),
                )
                .arg(
                    Arg::with_name("patch")
                        .long("patch")
                        .help("Print the PATCH version of this package."),
                )
                .arg(
                    Arg::with_name("pre")
                        .long("pre")
                        .help("Print the PRE-RELEASE version of this package."),
                )
                .arg(
                    Arg::with_name("build")
                        .long("build")
                        .help("Print the BUILD version of this package."),
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
        ("bump", Some(bump_matches)) => {
            bump(matches.value_of("manifest-path").unwrap(), bump_matches)
        }
        ("read", Some(read_matches)) => {
            read(matches.value_of("manifest-path").unwrap(), read_matches)
        }
        (_, _) => panic!("Unreachable - at least one subcommand must be specified."),
    };
}
