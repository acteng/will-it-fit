use std::path::PathBuf;

use anyhow::{anyhow, Result};

/// Obtains a path to a test file (test code only!)
/// This is a convenience function for writing test code. It allow tests code from anywhere in the
/// workspace to access test files (eg input .osm files, golden outputfiles etc) which are stored within
/// the `tests` package.
/// This function make direct reference to the location of this source file (using the `file!()` marco)
/// and hence should only be used in test code and not in any production code.
// Copied from here:
// https://github.com/a-b-street/abstreet/blob/d30c36a22a87824d3581e0d0d4e2faf9788d6176/tests/src/lib.rs#L29
pub fn get_test_file_path(path: String) -> Result<String> {
    // Get the absolute path to the crate that called was invoked at the cli (or equivalent)
    let maybe_workspace_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let maybe_workspace_dir = std::path::Path::new(&maybe_workspace_dir);
    // Get the relative path to this source file within the workspace
    let this_source_file = String::from(file!());

    // Try a find a suitable way to join the two paths to find something that exists
    let test_file = next_test_file_path(maybe_workspace_dir, &this_source_file);
    if test_file.is_ok() {
        // Now try and match the workspace path with the user requested path
        match next_test_file_path(test_file.as_ref().unwrap(), &path) {
            Ok(pb) => Ok(String::from(pb.to_str().unwrap())),
            Err(e) => Err(e),
        }
    } else {
        panic!("Cannot find the absolute path to {}. Check that this function being called from test code, not production code.", this_source_file);
    }
}

fn next_test_file_path(
    maybe_absolute_dir: &std::path::Path,
    file_path: &String,
) -> Result<PathBuf> {
    let path_to_test = maybe_absolute_dir.join(file_path);
    if path_to_test.exists() {
        Ok(path_to_test)
    } else if maybe_absolute_dir.parent().is_some() {
        next_test_file_path(maybe_absolute_dir.parent().unwrap(), file_path)
    } else {
        Err(anyhow!("Cannot locate file '{}'", file_path))
    }
}
