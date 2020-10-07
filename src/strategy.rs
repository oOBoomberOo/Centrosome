use std::io::Read;
use std::str::{from_utf8, Utf8Error};

pub trait Resource: Read {}

pub trait Content {
    fn as_bytes(&self) -> &[u8];

    fn as_str(&self) -> Result<&str, Utf8Error> {
        from_utf8(self.as_bytes())
    }
}

#[cfg(test)]
pub mod tests {}
