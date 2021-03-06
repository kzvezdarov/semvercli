//! Command line utility for setting, bumping, and reading Rust pacakge versions.
//! This is an extremely thin layer over the [semver crate](https://crates.io/crates/semver)
//! and meant to just serve as a command line glue for tools such as [cargo-make](https://crates.io/crates/cargo-make).
#[macro_use]
extern crate clap;
extern crate semver;
extern crate toml_edit;

#[cfg(test)]
extern crate tempfile;

use std::convert::TryFrom;
use std::fs;
use std::io;
use std::io::Write;
use std::ops::Deref;

use clap::{App, AppSettings, Arg, ArgGroup, ArgMatches, SubCommand};
use semver::{Identifier, Version};
use toml_edit::{value, Document};

fn parser<'a, 'b>() -> App<'a, 'b> {
    App::new("semvercli")
        .version(crate_version!())
        .settings(&[AppSettings::SubcommandRequiredElseHelp])
        .subcommand(
            SubCommand::with_name("read")
                .about("Read and print a specific version component.")
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
                .about("Bump or set a specific version component.")
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
                        .help("Set the full VERSION")
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
}

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
            Version::parse(&format!("0.0.0-{}", meta)).unwrap().pre,
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

fn write_manifest(manifest: Document, path: &str) {
    fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
        .expect("Could not find Cargo.toml to write to.")
        .write_all(manifest.to_string().as_bytes())
        .expect("Failed to write updated manifest to Cargo.toml");
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
fn read(manifest: &Document, matches: &ArgMatches) -> String {
    let version = read_version(manifest);

    if matches.is_present("major") {
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
    }
}

/// Bumps the package version string of the provided manifest;
/// panics if an incorrect pre-release/build/version string is
/// passed in the argument matches; assumes that it will always
/// be called with a component to bump.
fn bump(manifest: &mut Document, matches: &ArgMatches) {
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
}

/// Main entrypoint, which executes either a read or a bump depending on
/// the provided arguments. It takes in an output explicitly in order to
/// simplify testing.
fn execute(matches: &ArgMatches, stdout: &mut Write) {
    let manifest_path = matches.value_of("manifest-path").unwrap();
    let mut manifest = read_manifest(manifest_path);

    match matches.subcommand() {
        ("bump", Some(bump_matches)) => {
            bump(&mut manifest, bump_matches);
            write_manifest(manifest, manifest_path)
        }
        ("read", Some(read_matches)) => {
            let component = read(&manifest, read_matches);
            writeln!(stdout, "{}", component).unwrap();
        }
        (_, _) => panic!("Unreachable - at least one subcommand must be specified."),
    };
}

fn main() {
    let matches = parser().get_matches();

    execute(&matches, &mut io::stdout());
}


/// Property tests to validate read/bump behavior;
/// these are probably a massiver overkill given the simplicity of the implementation above,
/// but it's a useful demonstration/exercise in using proptest.
///
/// The current tests only ensure that the correct operation is performed given valid input for
/// the corresponding subcommand and work almost end to end - both start with a valid command line
/// input and assert on the final file/stream outputs (i.e. the `bump` makes use of tempdir, and `read`
/// just passes in a vector for stdout). It would be nice to eventually lift and combine those and demonstrate
/// verifying the full command from start to finish against more fuzzed inputs.
#[cfg(test)]
mod test {
    use proptest::option::of;
    use proptest::prelude::*;
    use toml_edit::{Document, Item, Table, value};
    use semver::{Version, Identifier};
    use tempfile::tempdir;

    use std::convert::TryFrom;
    use std::fs::File;
    use std::str;

    use super::*;

    /// Enum of operations that can be performed
    /// by each subcommand; generating the CLI
    /// convenience
    #[derive(Debug, Clone)]
    enum Op {
        Major,
        Minor,
        Patch,
        Pre(String),
        Build(String),
        Version(String)
    }

    prop_compose! {
        /// Metadata generation strategy that outputs semver parsed metadata labels. Both the
        /// build and prerelease labels conform to the same format, so only one strategy is needed.
        // Proptest doesn't seem to support the character classes from the regex crate, such as
        // the [[:alphanum:]] class
        fn metadata_strat()(label in r"[a-zA-Z0-9]+(\.[a-zA-Z0-9]+)*") -> Vec<Identifier> {
            dbg!(label.clone());
            VersionMetadata::try_from(label.as_str()).unwrap().0
        }
    }

    prop_compose! {
        /// Semantic version genration strategy; the only interesting thing here is making use
        /// of `proptest::option::of` to encode that build and prerelease labels are not always
        /// present.
        fn version_strat()(major in any::<u64>(),
                     minor in any::<u64>(),
                     patch in any::<u64>(),
                     pre in of(metadata_strat()),
                     build in of(metadata_strat())) -> Version {
            Version {
                major, minor, patch,
                pre: pre.unwrap_or(vec![]),
                build: build.unwrap_or(vec![])
            }
        }
    }

    prop_compose! {
        /// Manifest generation strategy; currently it will always be valid, in the
        /// sense of being valid TOML (since the output is a `toml_edit::Document`) and
        /// containing a `[package]` table which has a `version` key that maps to a
        /// valid semantic version string.
        fn manifest_strat()(version in version_strat()) -> Document {
            let mut manifest = Document::new();
            manifest["package"] = Item::Table(Table::new());
            manifest["package"]["version"] = value(version.to_string());

            return manifest
        }
    }

    /// Simple operation generation strategy that is shared by tests to both `bump`
    /// and `read`; in the case of `bump` the values for metadata labels and version
    /// are simply ignored.
    fn op_strat() -> impl Strategy<Value = Op> {
        prop_oneof![
            Just(Op::Major),
            Just(Op::Minor),
            Just(Op::Patch),
            metadata_strat()
                .prop_map(|p| Op::Pre(String::from(VersionMetadata(p)))),
            metadata_strat()
                .prop_map(|b| Op::Build(String::from(VersionMetadata(b)))),
            version_strat()
                .prop_map(|v| Op::Version(v.to_string()))
        ]
    }

    proptest! {
        /// Tests that given valid inputs to the bump subcommand and a valid manifest file
        /// the manifest file is updated with the correct version string.
        /// The test works by essentially reimplementing the logic of `bump` as minimalistically as possible -
        /// it just matches each op to an effect, and using a temp. file to validate the file manipulation performed
        /// by the subcomand.
        #[test]
        fn test_bump(manifest in manifest_strat(), op in op_strat()) {
            let tmpdir = tempdir().unwrap();
            let tmp_path = tmpdir.path().join("Cargo.toml");
            let manifest_path = tmp_path.to_str().unwrap();
            File::create(tmp_path.clone()).unwrap();

            let old_version = read_version(&manifest);

            let mut cli_args = vec!["version-bump",
                                    "--manifest-path",
                                    manifest_path,
                                    "bump"];

            cli_args.extend_from_slice(
                match op {
                    Op::Major => vec!["--major"],
                    Op::Minor => vec!["--minor"],
                    Op::Patch => vec!["--patch"],
                    Op::Pre(ref pre) => vec!["--pre", pre.as_str()],
                    Op::Build(ref build) => vec!["--build", build.as_str()],
                    Op::Version(ref version) => vec!["--version", version.as_str()],
                }.as_slice());

            write_manifest(manifest, manifest_path);

            let matches = parser().get_matches_from(cli_args.as_slice());
            let mut stdout = Vec::new();

            execute(&matches, &mut stdout);

            let bumped_manifest = read_manifest(manifest_path);
            let bumped_version = read_version(&bumped_manifest);


            match op {
                Op::Major => assert_eq!(old_version.major + 1, bumped_version.major),
                Op::Minor => assert_eq!(old_version.minor + 1, bumped_version.minor),
                Op::Patch => assert_eq!(old_version.patch + 1, bumped_version.patch),
                Op::Pre(pre) => assert_eq!(pre,
                                           String::from(VersionMetadata(bumped_version.pre))),
                Op::Build(build) => assert_eq!(build,
                                               String::from(VersionMetadata(bumped_version.build))),
                Op::Version(version) => assert_eq!(version, bumped_version.to_string()),
            };
        }

        /// Tests that given valid inputs to read the correct version component is written
        /// to `stdout`. It does so by reimplementing the minimum amount of logic from `read` to
        /// parse the component out of the input version and compare to what was written to `stdout`.
        #[test]
        fn test_read(manifest in manifest_strat(), op in op_strat()) {
            let tmpdir = tempdir().unwrap();
            let tmp_path = tmpdir.path().join("Cargo.toml");
            let manifest_path = tmp_path.to_str().unwrap();
            File::create(tmp_path.clone()).unwrap();

            let version = read_version(&manifest);

            let mut cli_args = vec!["version-bump",
                                    "--manifest-path",
                                    manifest_path,
                                    "read"];

            cli_args.extend_from_slice(
                match op {
                    Op::Major => &["--major"],
                    Op::Minor => &["--minor"],
                    Op::Patch => &["--patch"],
                    Op::Pre(_) => &["--pre"],
                    Op::Build(_) => &["--build"],
                    Op::Version(_) => &["--version"]
                });

            write_manifest(manifest, manifest_path);

            let matches = parser().get_matches_from(cli_args.as_slice());
            let mut stdout = Vec::new();

            execute(&matches, &mut stdout);

            let expected = match op {
                Op::Major => format!("{}\n", version.major),
                Op::Minor => format!("{}\n", version.minor),
                Op::Patch => format!("{}\n", version.patch),
                Op::Pre(_) => format!("{}\n",
                                      String::from(VersionMetadata(version.pre))),
                Op::Build(_) => format!("{}\n",
                                        String::from(VersionMetadata(version.build))),
                Op::Version(_) => format!("{}\n",
                                          version.to_string())
            };

            assert_eq!(str::from_utf8(&stdout).unwrap(), expected.as_str());
        }
    }
}
