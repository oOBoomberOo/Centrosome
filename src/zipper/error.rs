use std::path::PathBuf;

#[derive(Debug)]
pub enum ZipperError {
	Io(std::io::Error),
	UnableToConvertDatapack,
	UnknownFormat(PathBuf),
}

use std::error;
use std::fmt;

impl fmt::Display for ZipperError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ZipperError::Io(error) => write!(f, "{}", error),
			ZipperError::UnableToConvertDatapack => write!(f, "Unable to convert to datapack"),
			ZipperError::UnknownFormat(path) => {
				write!(f, "Unknown file format: {}", path.display())
			}
		}
	}
}

impl error::Error for ZipperError {
	fn description(&self) -> &str {
		match *self {
			ZipperError::Io(ref io_err) => (io_err as &dyn error::Error).description(),
			ZipperError::UnableToConvertDatapack => "Unable to convert to datapack",
			ZipperError::UnknownFormat(_) => "Unknown file format",
		}
	}

	fn cause(&self) -> Option<&dyn error::Error> {
		match *self {
			ZipperError::Io(ref io_err) => Some(io_err as &dyn error::Error),
			_ => None,
		}
	}
}

use std::convert;
use std::io;

impl convert::From<ZipperError> for io::Error {
	fn from(err: ZipperError) -> io::Error {
		io::Error::new(io::ErrorKind::Other, err)
	}
}