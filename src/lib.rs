//! Parse a cargo workspace and check that all packages have `lints.workspace=true` set.

use cargo_metadata::{MetadataCommand, PackageId};
use std::{collections::HashSet, fs};
use std::{error, fmt, io};

/// Validate that all packages in the workspace have `lints.workspace = true`.
///
/// # Arguments
/// * `metadata_command`: The command to run to generate metadata.
/// * `verbose`: If set to true, provides more detailed output to stderr.
///
/// # Errors
/// If the validation fails, it returns an error indicating the kind of failure. The failure may
/// indicate I/O-related failures to read and parse data, or it may indicate that individual
/// packages do not have `lints.workspace = true`.
pub fn validate_workspace(
    metadata_command: &MetadataCommand,
    verbose: bool,
) -> Result<(), WorkspaceValidationError> {
    let metadata = metadata_command.exec()?;
    let workspace_members = metadata
        .workspace_members
        .into_iter()
        .collect::<HashSet<_>>();
    let mut failing_packages = Vec::new();
    for package in metadata.packages {
        // Skip anything not in the workspace
        if !workspace_members.contains(&package.id) {
            continue;
        }
        let manifest_path = package.manifest_path.as_path();
        let manifest: toml::Table = toml::from_str(&fs::read_to_string(manifest_path)?)?;
        if let Err(kind) = validate_package(&package, &manifest, verbose) {
            failing_packages.push(PackageValidationError {
                kind,
                package: package.id,
            });
        }
    }
    if failing_packages.is_empty() {
        Ok(())
    } else {
        Err(WorkspaceValidationError::FailingPackages(failing_packages))
    }
}

/// Validate that the given package has `lints.workspace = true`.
///
/// # Arguments
/// * `package`: The package details, as returned by [`cargo_metadata`].
/// * `manifest`: The `Cargo.toml` manifest for this package, parsed as `toml`.
/// * `verbose`: If set to true, provides more detailed output to stderr.
///
/// # Errors
/// If the validation fails, it returns an error indicating the kind of failure.
pub fn validate_package(
    package: &cargo_metadata::Package,
    manifest: &toml::Table,
    verbose: bool,
) -> Result<(), PackageValidationErrorKind> {
    match manifest
        .get("lints")
        .and_then(|lints| lints.get("workspace"))
    {
        Some(toml::Value::Boolean(true)) => {
            if verbose {
                eprintln!(
                    "PASS: Package {} ({})",
                    package.name,
                    package.manifest_path.as_str()
                );
            }
            Ok(())
        }
        Some(other_value) => {
            if verbose {
                eprintln!(
                    "FAIL: Package {} ({}) has `lints.workspace = {other_value}`",
                    package.name,
                    package.manifest_path.as_str()
                );
            }
            Err(PackageValidationErrorKind::WorkspaceLintsWrongValue(
                other_value.clone(),
            ))
        }
        None => {
            if verbose {
                eprintln!(
                    "FAIL: Package {} ({}) missing `lints.workspace` field",
                    package.name,
                    package.manifest_path.as_str()
                );
            }
            Err(PackageValidationErrorKind::WorkspaceLintsMissing)
        }
    }
}

/// All the reasons why we might fail a workspace.
#[derive(Debug)]
pub enum WorkspaceValidationError {
    /// IO error.
    Io(io::Error),
    /// Error running `cargo metadata`.
    CargoMetadata(cargo_metadata::Error),
    /// Error parsing `Cargo.toml` manifest as TOML
    Toml(toml::de::Error),
    /// Packages successfully read but failed the check.
    FailingPackages(Vec<PackageValidationError>),
}
impl From<io::Error> for WorkspaceValidationError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
impl From<cargo_metadata::Error> for WorkspaceValidationError {
    fn from(error: cargo_metadata::Error) -> Self {
        Self::CargoMetadata(error)
    }
}
impl From<toml::de::Error> for WorkspaceValidationError {
    fn from(error: toml::de::Error) -> Self {
        Self::Toml(error)
    }
}
impl fmt::Display for WorkspaceValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => f.write_fmt(format_args!(
                "Disk I/O Error reading `Cargo.toml` files:\n    {e}\n"
            )),
            Self::CargoMetadata(e) => f.write_fmt(format_args!(
                "Error reading Cargo manifest data:\n    {e}\n"
            )),
            Self::Toml(e) => f.write_fmt(format_args!(
                "Error parsing `Cargo.toml` files as TOML:\n    {e}\n"
            )),
            Self::FailingPackages(package_failures) => {
                f.write_str("Failing packages:")?;
                for failure in package_failures {
                    f.write_fmt(format_args!("\n* {failure}"))?;
                }
                Ok(())
            }
        }
    }
}

/// A package failed the check.
#[derive(Debug)]
pub struct PackageValidationError {
    /// Why the package failed.
    kind: PackageValidationErrorKind,
    /// Which package failed.
    package: PackageId,
}

impl fmt::Display for PackageValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "Package {}:\n     {}\n",
            self.package, self.kind
        ))
    }
}
impl error::Error for PackageValidationError {}

/// Why a package might fail the check.
#[derive(Debug)]
pub enum PackageValidationErrorKind {
    /// There was no `lints.workspace` field.
    WorkspaceLintsMissing,
    /// The `lints.workspace` field was provided, but had the wrong value.
    WorkspaceLintsWrongValue(toml::Value),
}
impl fmt::Display for PackageValidationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorkspaceLintsMissing => f.write_str("No `workspace.lints` field found"),
            Self::WorkspaceLintsWrongValue(found) => {
                f.write_fmt(format_args!("workspace.lints = {found}, expected `true`"))
            }
        }
    }
}
