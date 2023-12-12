//! Cache for downloaded files

use std::{
    fs::remove_file,
    hash::Hasher,
    path::{Path, PathBuf},
};

use http::StatusCode;
use log::{debug, warn};
use rs_sha512::{HasherContext, Sha512Hasher};

use crate::{
    error::{Error, ErrorExt},
    util::{
        self, download,
        fs::{copy, rename},
    },
};

/// A download cache
pub struct DownloadCache {
    /// The directory to use for caching
    workdir: PathBuf,
}

impl DownloadCache {
    /// Creates a new download cache at the supplied location
    ///
    /// This function will ensure the directory does exist
    /// # Arguments
    /// * `workdir` - The directory to use for caching
    pub fn new(workdir: PathBuf) -> Result<Self, Error> {
        util::fs::create_dir_all(&workdir).e_context(|| {
            format!(
                "Creating new download cache at {}",
                workdir.to_string_lossy()
            )
        })?;
        Ok(Self { workdir })
    }

    /// Downloads a url through the cache by hashing the `url` and checking for available cached files
    ///
    /// Uses the [download::download_to_file()] function
    /// # Arguments
    /// * `url` - The URL to fetch from
    /// * `file` - The file to download to
    /// * `message` - The message to log when downloading
    /// * `expect_success` - If this function should return an error if a non-ok status code is encountered
    /// # Errors
    /// - If the `expect_success` option is set to `true`, this function will error on a non-ok status
    /// - If an unknown HTTP response status is received
    /// - Any CURL error
    pub fn download(
        &self,
        url: &str,
        file: &Path,
        message: &str,
        expect_success: bool,
    ) -> Result<StatusCode, Error> {
        let hash = hash_string(url);

        let cache_path = self.workdir.join(&hash);
        if cache_path.exists() {
            debug!("Using cached value {hash}");

            match copy(&cache_path, file) {
                Ok(_) => Ok(StatusCode::OK),
                Err(e) => {
                    warn!("Couldn't use cache for {}: {} - DROPPING", url, e);

                    remove_file(cache_path)
                        .e_context(|| format!("Dropping cached value {} for {}", hash, url))?;

                    download::download_to_file(url, file, message, expect_success)
                }
            }
        } else {
            // Download the file to a temporary path
            let temp_path = self.workdir.join(format!("{}_temp", &hash));
            let res = download::download_to_file(url, &temp_path, message, expect_success)?;

            if res.is_success() {
                debug!("Creating cached value {hash}");

                rename(&temp_path, &cache_path)
                    .e_context(|| format!("Creating cache value {} for {}", hash, url))?;

                copy(&cache_path, file)
                    .e_context(|| format!("Using cache value {} for {}", hash, url))?;
            } else {
                remove_file(cache_path)
                    .e_context(|| format!("Dropping cached value {} for {}", hash, url))?;
            }
            Ok(res)
        }
    }
}

/// Hashes the supplied string
fn hash_string(string: &str) -> String {
    let mut hasher = Sha512Hasher::default();
    hasher.write(string.as_bytes());
    let bytes_result = HasherContext::finish(&mut hasher);
    format!("{bytes_result:02x}")
}
