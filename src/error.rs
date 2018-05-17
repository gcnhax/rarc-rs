use std::io;
use std::fmt;
use std::error::Error as StdError;
use nom;

#[derive(Debug)]
pub enum Error {
    /// An I/O error encountered when reading or writing a file or cursor during RARC manipulation.
    Io(io::Error),
    /// A parse error encountered when attempting to parse RARC metadata.
    Parse(nom::Err),

    /// Encountered if no nodes are present in the RARC node table.
    NoNodes,
    /// Encountered if the first node in the RARC table does not have shortname `ROOT`.
    NoRootNode,
    /// Encountered if decoding a filename (as shift_jis) from the string table errors.
    NameEncodingError(String),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<nom::ErrorKind> for Error {
    fn from(err: nom::ErrorKind) -> Error {
        Error::Parse(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::Io(io_err) => write!(f, "IO error: {}", io_err),
            Error::Parse(parse_err) => write!(f, "Parse error: {}", parse_err),
            Error::NameEncodingError(err) => write!(f, "Error encoding filename: {}", err),
            _ => f.write_str(self.description()),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref io_err) => io_err.description(),
            Error::Parse(ref parse_err) => parse_err.description(),
            Error::NameEncodingError(_) => "Error decoding filename",
            Error::NoNodes => "No nodes present in node table",
            Error::NoRootNode => "First node found in node table is not ROOT",
        }
    }
}
