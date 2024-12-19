//! Error handling.

/// cdtest errors.
#[derive(Clone)]
pub enum Error {
    InitializeVarFailed,
    InitializeTmpFailed,
    ParseFailedTime { from: String },
    ParseFailedToml { from: String },
    ParseFailedPath { from: String },
    ProjectDirectoryInvalid,
    ProjectSetupFailed,
    SubprocessFailed,
    WriteOutFailed,
}

impl Error {
    fn to_string(&self) -> String {
        match self {
            Error::InitializeVarFailed => "Failed to initialize /var/tmp/cdtest".into(),
            Error::InitializeTmpFailed => "Failed to initialize /tmp/cdtest".into(),
            Error::ParseFailedTime { from } => format!("Failed to parse time from `{}`", from),
            Error::ParseFailedToml { from } => format!("Failed to parse toml from `{}`", from),
            Error::ParseFailedPath { from } => format!("Failed to parse path from `{}`", from),
            Error::ProjectDirectoryInvalid => "The project directory is invalid".into(),
            Error::ProjectSetupFailed => "Project failed to set up".into(),
            Error::SubprocessFailed => "Subprocess failed to execute correctly".into(),
            Error::WriteOutFailed => "Failed to write out project".into(),
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("Error")
            .field("message", &self.to_string())
            .finish()
    }
}
