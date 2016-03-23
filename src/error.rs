use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fmt::Result as FmtResult;
use std::io;

use fmt::Format;

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
#[allow(dead_code)]
pub enum CliErrorKind {
    UnknownBoolArg,
    TomlTableRoot,
    CurrentDir,
    Unknown,
    Io(io::Error),
    Generic(String),
}

impl CliErrorKind {
    fn description(&self) -> &str {
        match *self {
            CliErrorKind::Generic(ref e) => e,
            CliErrorKind::TomlTableRoot => "No root table found for toml file",
            CliErrorKind::CurrentDir => "Unable to determine the current working directory",
            CliErrorKind::UnknownBoolArg => "The value supplied isn't valid, either use 'true/false', 'yes/no', or the first letter of either.",
            CliErrorKind::Unknown => "An unknown fatal error has occurred, please consider filing a bug-report!",
            CliErrorKind::Io(ref e) => e.description(),
        }
    }
}

impl From<CliErrorKind> for CliError {
    fn from(kind: CliErrorKind) -> Self {
        CliError {
            error: format!("{} {}", Format::Error("error:"), kind.description()),
            kind: kind,
        }
    }
}

#[derive(Debug)]
pub struct CliError {
    /// The formatted error message
    pub error: String,
    /// The type of error
    pub kind: CliErrorKind,
}

// Copies clog::error::Error;
impl CliError {
    /// Return whether this was a fatal error or not.
    pub fn use_stderr(&self) -> bool {
        // For now all errors are fatal
        true
    }

    /// Print this error and immediately exit the program.
    ///
    /// If the error is non-fatal then the error is printed to stdout and the
    /// exit status will be `0`. Otherwise, when the error is fatal, the error
    /// is printed to stderr and the exit status will be `1`.
    pub fn exit(&self) -> ! {
        if self.use_stderr() {
            wlnerr!("{}", self);
            ::std::process::exit(1)
        }
        println!("{}", self);
        ::std::process::exit(0)
    }
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.error)
    }
}

impl Error for CliError {
    fn description(&self) -> &str {
        self.kind.description()
    }

    fn cause(&self) -> Option<&Error> {
        match self.kind {
            CliErrorKind::Io(ref e) => Some(e),
            _ => None
        }
    }
}

impl From<io::Error> for CliError {
    fn from(ioe: io::Error) -> Self {
        CliError {
            error: format!("{} {}", Format::Error("error:"), ioe.description()),
            kind: CliErrorKind::Io(ioe),
        }
    }
}
