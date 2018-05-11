use std::io;
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
