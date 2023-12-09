//! Utilities for handling strings

use crate::package::CorePackage;

/// Substitutes the following strings:
/// - `$PKG_NAME`: Package name
/// - `$PKG_VERSION`: Package version
/// - `$PKG_ARCH`: Package architecture
///
/// with the values of the supplied package
/// # Arguments
/// * `string` - The string to replace in
/// * `package` - The package to pull the variables from
pub fn replace_package_variables(string: &str, package: &dyn CorePackage) -> String {
    string
        .replace("$PKG_NAME", package.get_name())
        .replace("$PKG_VERSION", package.get_version())
        .replace("$PKG_ARCH", package.get_arch())
}
