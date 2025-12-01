use thiserror::Error;

pub use semver::Error as SemverError;
pub use std::io::Error as StdIoError;
pub use std::process::Output as StdProcessOutput;
pub use std::string::FromUtf8Error as StdStringFromUtf8Error;

#[derive(Debug, Error)]
/// Error kinds of what can go wrong when a compilation is invoked
pub enum Error {
    #[error("IO Error: {0}")]
    IO(StdIoError),
    #[error("No Success: {0:?}")]
    Unsuccesful(StdProcessOutput),
}

#[derive(Debug, Error)]
/// Error kinds of what can go wrong when the version is requested
pub enum VersionError {
    #[error("IO Error: {0}")]
    IO(StdIoError),
    #[error("Invocation no success: {0:?}")]
    InvocationNoSuccess(StdProcessOutput),
    #[error("Attempt read stdout failed: {0}")]
    AttemptReadStdOut(StdStringFromUtf8Error),
    #[error("Regex no match: {0}")]
    RegexNoMatch(String),
    #[error("Version parse failed: {0}")]
    VersionParseFailed(SemverError),
}
