//! Utilities for downloading files
use http::StatusCode;
use log::info;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use curl::easy::Easy;

use crate::error::support::CURLError;
use crate::error::Error;
use crate::error::ErrorExt;
use crate::error::ErrorType;
use crate::error::Throwable;

/// Downloads the contents of the supplied url to the supplied file
/// # Arguments
/// * `url` - The URL to fetch from
/// * `file` - The file to download to
/// * `message` - The message to log when downloading
/// * `expect_success` - If this function should return an error if a non-ok status code is encountered
/// # Errors
/// - If the `expect_success` option is set to `true`, this function will error on a non-ok status
/// - If an unknown HTTP response status is received
/// - Any CURL error
pub fn download_to_file(
    url: &str,
    file: &Path,
    message: &str,
    expect_success: bool,
) -> Result<StatusCode, Error> {
    let context = || format!("Downloading {} to {}", url, file.to_string_lossy());

    let mut file = File::create(file).e_context(context)?;

    download(url, message, expect_success, move |data| {
        file.write_all(data).is_ok()
    })
    .e_context(context)
}

/// Downloads the contents of the supplied url
/// # Arguments
/// * `url` - The URL to fetch from
/// * `message` - The message to log when downloading
/// * `expect_success` - If this function should return an error if a non-ok status code is encountered
/// * `write_function` - The callback to use for writing
/// # Errors
/// - If the `expect_success` option is set to `true`, this function will error on a non-ok status
/// - If an unknown HTTP response status is received
/// - Any CURL error
pub fn download<'data, F>(
    url: &str,
    message: &str,
    expect_success: bool,
    mut write_function: F,
) -> Result<StatusCode, Error>
where
    F: FnMut(&[u8]) -> bool + Send + 'data,
{
    let context = || message.to_owned();

    //Create the curl context and set the url
    let mut easy = Easy::new();
    easy.url(url).e_context(context)?;

    //Allow CURL to follow redirections
    easy.follow_location(true).e_context(context)?;

    //Setup the low speed bounds (less that 1000bytes in 30 seconds)
    easy.low_speed_limit(1000).e_context(context)?;
    easy.low_speed_time(Duration::from_secs(30))
        .e_context(context)?;

    let transfer_res = {
        //Create a scoped transfer and perform it
        let mut transfer = easy.transfer();
        transfer
            .write_function(move |data| match write_function(data) {
                true => Ok(data.len()),
                false => Ok(data.len() - 1),
            })
            .e_context(context)?;

        info!("{}", message);

        //Perform now
        transfer.perform()
    };

    match transfer_res {
        Ok(_) => {
            let code = easy.response_code().e_context(context)?;

            let status = match StatusCode::from_u16(code as u16) {
                Ok(status) => status,
                Err(_) => return Err(Error::new(ErrorType::CURL(CURLError::InvalidStatus(code)))),
            };

            if expect_success {
                if !status.is_success() {
                    Err(Error::new(ErrorType::CURL(CURLError::ErrorStatus(status))))
                } else {
                    Ok(status)
                }
            } else {
                Ok(status)
            }
        }
        Err(e) => Err(e.throw(message.to_owned())),
    }
}
