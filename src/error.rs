use std::fmt;
use std::io;
use std::str;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    IncorrectSegyFormat(),
    UnsupportedEndianness(u32),
    UnsupportedFormatCode(i16),
    Utf8(str::Utf8Error),
    UnsupportedNumberOfStanzas(i32),
    NoSuchHeader(i32),
    TraceHeaderMap(i32),
    ZeroSampleInterval(),
    ZeroSampleNumber(),
    WrongFormatForRevision(i16),
    UnsupportedRevision(u8),
    DiffDimToWrite(),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::IncorrectSegyFormat() => write!(f, "Incorrect SEGY format."),
            Error::UnsupportedEndianness(v) => {
                write!(f, "Unsupported endianness in binary header: {:X}", v)
            }
            Error::UnsupportedFormatCode(v) => {
                write!(f, "Unsupported format code in binary header: {}", v)
            }
            Error::Utf8(err) => {
                write!(
                    f,
                    "Data in extended text header isn't a utf-8 text: {}",
                    err
                )
            }
            Error::UnsupportedNumberOfStanzas(v) => {
                write!(f, "Unsupported number of trailer stanzas: {}", v)
            }
            Error::NoSuchHeader(hdr_name) => {
                write!(
                    f,
                    "Requested trace header doesn't exist in trace header map: {}",
                    hdr_name
                )
            }
            Error::TraceHeaderMap(hdr_name) => {
                write!(
                    f,
                    "Requested trace header on offset more than in data: {}",
                    hdr_name
                )
            }
            Error::ZeroSampleInterval() => {
                write!(f, "Sample interval in binary header can not be null.",)
            }
            Error::ZeroSampleNumber() => {
                write!(f, "Sample number in binary header can not be null.",)
            }
            Error::WrongFormatForRevision(fmt) => {
                write!(f, "The format {} can not be used in chosen revision.", fmt)
            }
            Error::UnsupportedRevision(rev) => {
                write!(f, "The revision {} is not supported.", rev)
            }
            Error::DiffDimToWrite() => {
                write!(f, "Size of vectors passed to write is different.")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(err: str::Utf8Error) -> Self {
        Error::Utf8(err)
    }
}
