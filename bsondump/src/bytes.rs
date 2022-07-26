use bson::{RawArray, RawBsonRef, RawDocument};

pub trait CountBytes {
    fn count_bytes(&self) -> usize;
}

impl CountBytes for &str {
    fn count_bytes(&self) -> usize {
        // i32 size + characters + null terminator
        4 + self.len() + 1
    }
}

impl CountBytes for RawDocument {
    fn count_bytes(&self) -> usize {
        self.as_bytes().len()
    }
}

impl CountBytes for RawArray {
    fn count_bytes(&self) -> usize {
        self.as_bytes().len()
    }
}

impl CountBytes for bson::RawBsonRef<'_> {
    fn count_bytes(&self) -> usize {
        match self {
            RawBsonRef::Double(_) => 8,
            RawBsonRef::String(string) => string.count_bytes(),
            RawBsonRef::Array(raw_array) => raw_array.count_bytes(),
            RawBsonRef::Document(raw_document) => raw_document.count_bytes(),
            RawBsonRef::Boolean(_) => 1,
            RawBsonRef::Null => 0,
            RawBsonRef::RegularExpression(regex) => {
                regex.pattern.count_bytes() + regex.options.count_bytes()
            }
            RawBsonRef::JavaScriptCode(code) => code.count_bytes(),
            RawBsonRef::JavaScriptCodeWithScope(cws) => {
                cws.code.count_bytes() + cws.scope.count_bytes()
            }
            RawBsonRef::Int32(_) => 4,
            RawBsonRef::Int64(_) => 8,
            RawBsonRef::Timestamp(_) => 8,
            RawBsonRef::Binary(raw_binary_ref) => 4 + 1 + raw_binary_ref.bytes.len(),
            RawBsonRef::ObjectId(_) => 12,
            RawBsonRef::DateTime(_) => 8,
            RawBsonRef::Symbol(symbol) => symbol.count_bytes(),
            RawBsonRef::Decimal128(dec) => dec.bytes().len(),
            RawBsonRef::Undefined => 0,
            RawBsonRef::MaxKey => 0,
            RawBsonRef::MinKey => 0,
            RawBsonRef::DbPointer(_) => "".count_bytes() + 12,
        }
    }
}
