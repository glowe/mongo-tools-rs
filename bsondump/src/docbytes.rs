use std::{io::Read, result::Result};

pub struct BsonBytes {
    pub size: u32,
    pub bytes: Vec<u8>,
}
pub struct Source<'reader, R: Read> {
    reader: &'reader mut R,
}

pub fn source<R: Read>(reader: &mut R) -> Source<R> {
    Source { reader }
}

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    TooSmallError(u8),
    TooLargeError(u32),
}

// FIXME: This is a bsondump limitation that has to do with mongodb, bson has no maxium size
// 16kb + 16mb - This is the maximum size we would get when dumping the
// oplog itself. See https://jira.mongodb.org/browse/TOOLS-3001.
const MAX_BSON_SIZE: u32 = (16 * 1024 * 1024) + (16 * 1024);


impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::TooSmallError(bson_size) => write!(
                f,
                "invalid BSONSize: {} bytes is less than {} bytes",
                bson_size, MIN_BSON_SIZE
            ),
            Error::TooLargeError(bson_size) => write!(
                f,
                "invalid BSONSize: {} bytes is larger than than maximum of {} bytes",
                bson_size, MAX_BSON_SIZE
            ),

            Error::IOError(ref err) => err.fmt(f),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

// 4 bytes for the size + 1 byte for the null terminator
const MIN_BSON_SIZE: u32 = 5;

impl<'r, R: Read> std::iter::Iterator for Source<'r, R> {
    type Item = Result<BsonBytes, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut size_bytes: [u8; 4] = [0, 0, 0, 0];
        if let Err(err) = self.reader.read_exact(&mut size_bytes) {
            if let std::io::ErrorKind::UnexpectedEof = err.kind() {
                return None;
            } else {
                return Some(Err(Error::IOError(err)));
            }
        }
        let size = i32::from_le_bytes(size_bytes) as u32;

        if size < MIN_BSON_SIZE {
            return Some(Err(Error::TooSmallError(size as u8)));
        }

        if size > MAX_BSON_SIZE {
            return Some(Err(Error::TooLargeError(size)));
        }

        let mut remainder: Vec<u8> = vec![0u8; size as usize - size_bytes.len()];
        if let Err(err) = self.reader.read_exact(&mut remainder) {
            return Some(Err(Error::IOError(err)));
        }

        let mut raw_data = Vec::with_capacity(size as usize);
        raw_data.extend_from_slice(&size_bytes);
        raw_data.extend(&remainder);
        Some(Ok(BsonBytes { size, bytes: raw_data }))
    }
}
