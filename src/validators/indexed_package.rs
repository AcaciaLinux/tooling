//! Validators for `IndexedPackage`s
use super::{ValidationInput, ValidatorAction};
use crate::{error::Error, package::IndexedPackage, util::fs::ToPathBuf};
use std::{collections::LinkedList, path::PathBuf};

/// The validation result attached to a file
pub struct FileValidationResult<'a> {
    /// The path to the file that was validated
    pub path: PathBuf,
    /// The actions to take
    pub actions: Vec<ValidatorAction<'a>>,
    /// The errors that occured
    pub errors: Vec<Error>,
}

/// Validates a `IndexedPackage` by iterating over its index and validating everything
/// # Arguments
/// * `package` - The package to validate
/// * `input` - The validation input
/// # Returns
/// A vector of file results. If a file has no actions and no errors, it will not be returned
pub fn validate_indexed_package<'a>(
    package: &dyn IndexedPackage,
    input: &'a ValidationInput,
) -> Vec<FileValidationResult<'a>> {
    let mut res = Vec::new();

    package
        .get_index()
        .iterate(&mut LinkedList::new(), true, &mut |path, entry| {
            match entry {
                crate::util::fs::FSEntry::ELF(elf) => {
                    let path = path.to_path_buf().join(entry.name());
                    let v_res = elf.validate(input);

                    // If there are some results, append them to the return value
                    if !(v_res.actions.is_empty() && v_res.errors.is_empty()) {
                        res.push(FileValidationResult {
                            path,
                            actions: v_res
                                .actions
                                .into_iter()
                                .map(ValidatorAction::ELF)
                                .collect(),
                            errors: v_res.errors,
                        });
                    }
                }
                _ => {}
            }

            true
        });

    res
}
