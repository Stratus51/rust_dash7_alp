pub mod action_query;
pub mod break_query;
pub mod comparison_with_value;
pub mod define;

use super::super::error::QueryDecodeError;
use define::QueryCode;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Query<'item> {
    // NonVoid(non_void::NonVoid),
    // ComparisonWithZero(ComparisonWithZero),
    ComparisonWithValue(comparison_with_value::ComparisonWithValue<'item>),
    // ComparisonWithOtherFile(ComparisonWithOtherFile),
    // BitmapRangeComparison(BitmapRangeComparison),
    // StringTokenSearch(StringTokenSearch),
}

impl<'item> Query<'item> {
    /// Encodes the Item into a data pointer without checking the size of the
    /// receiving byte array.
    ///
    /// This method is meant to allow unchecked cross language wrapper libraries
    /// to implement an unchecked call without having to build a fake slice with
    /// a fake size.
    ///
    /// It is not meant to be used inside a Rust library/binary.
    ///
    /// # Safety
    /// You are responsible for checking that `out.len() >= size`. Failing that
    /// will result in the program writing out of bound. In the current
    /// implementation, it will silently attempt to write out of bounds.
    pub unsafe fn encode_in_ptr(&self, out: *mut u8) -> usize {
        match self {
            Self::ComparisonWithValue(query) => query.encode_in_ptr(out),
        }
    }

    /// Encodes the Item without checking the size of the receiving
    /// byte array.
    ///
    /// # Safety
    /// You are responsible for checking that `size` == [self.size()](#method.size) and
    /// to insure `out.len() >= size`. Failing that will result in the
    /// program writing out of bound. In the current implementation, it
    /// implementation, it will silently attempt to write out of bounds.
    pub unsafe fn encode_in_unchecked(&self, out: &mut [u8]) -> usize {
        self.encode_in_ptr(out.as_mut_ptr())
    }

    /// Encodes the value into pre allocated array.
    ///
    /// Fails if the pre allocated array is smaller than [self.size()](#method.size)
    /// returning the number of input bytes required.
    pub fn encode_in(&self, out: &mut [u8]) -> Result<usize, usize> {
        let size = self.size();
        if out.len() >= size {
            Ok(unsafe { self.encode_in_ptr(out.as_mut_ptr()) })
        } else {
            Err(size)
        }
    }

    /// Size in bytes of the encoded equivalent of the item.
    pub fn size(&self) -> usize {
        match self {
            Self::ComparisonWithValue(query) => query.size(),
        }
    }

    /// Creates a decodable item from a data pointer without checking the data size.
    ///
    /// This method is meant to allow unchecked cross language wrapper libraries
    /// to implement an unchecked call without having to build a fake slice with
    /// a fake size.
    ///
    /// It is not meant to be used inside a Rust library/binary.
    ///
    /// # Safety
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableVarint.size()](struct.DecodableVarint.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array.
    pub unsafe fn start_decoding_ptr<'data>(data: *const u8) -> Result<DecodableQuery<'data>, u8> {
        DecodableQuery::from_ptr(data)
    }

    /// Creates a decodable item without checking the data size.
    ///
    /// # Safety
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableVarint.size()](struct.DecodableQuery.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    ///
    /// Failing that might result in reading and interpreting data outside the given
    /// array.
    pub const unsafe fn start_decoding_unchecked(data: &[u8]) -> Result<DecodableQuery, u8> {
        DecodableQuery::new(data)
    }

    /// Creates a decodable item.
    ///
    /// This decodable item allows each parts of the item independently.
    pub fn start_decoding(data: &[u8]) -> Result<DecodableQuery, QueryDecodeError> {
        if data.is_empty() {
            return Err(QueryDecodeError::MissingBytes(1));
        }
        let ret = unsafe {
            Self::start_decoding_unchecked(data).map_err(QueryDecodeError::BadQueryCode)?
        };
        let ret_size = ret.size();
        if data.len() < ret_size {
            return Err(QueryDecodeError::MissingBytes(ret_size));
        }
        Ok(ret)
    }

    /// Decodes the Item from a data pointer.
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    ///
    /// This method is meant to allow unchecked cross language wrapper libraries
    /// to implement an unchecked call without having to build a fake slice with
    /// a fake size.
    ///
    /// It is not meant to be used inside a Rust library/binary.
    ///
    /// # Safety
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableVarint.size()](struct.DecodableVarint.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    ///
    /// Failing that will result in reading and interpreting data outside the given
    /// array.
    pub unsafe fn decode_ptr(data: *const u8) -> Result<(Self, usize), QueryDecodeError> {
        Ok(Self::start_decoding_ptr(data)
            .map_err(QueryDecodeError::BadQueryCode)?
            .complete_decoding())
    }

    /// Decodes the Item from bytes.
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    ///
    /// # Safety
    /// You are to check that data is not empty and that data.len() >=
    /// [DecodableQuery.size()](struct.DecodableQuery.html#method.size)
    /// (the expected byte size of the returned DecodableItem).
    ///
    /// Failing that will result in reading and interpreting data outside the given
    /// array.
    pub unsafe fn decode_unchecked(data: &'item [u8]) -> Result<(Self, usize), QueryDecodeError> {
        Ok(Self::start_decoding_unchecked(data)
            .map_err(QueryDecodeError::BadQueryCode)?
            .complete_decoding())
    }

    /// Decodes the item from bytes.
    ///
    /// On success, returns the decoded data and the number of bytes consumed
    /// to produce it.
    pub fn decode(data: &'item [u8]) -> Result<(Self, usize), QueryDecodeError> {
        Ok(Self::start_decoding(data)?.complete_decoding())
    }
}

pub enum DecodableQuery<'data> {
    // NonVoid(non_void::NonVoid),
    // ComparisonWithZero(ComparisonWithZero),
    ComparisonWithValue(comparison_with_value::DecodableComparisonWithValue<'data>),
    // ComparisonWithOtherFile(ComparisonWithOtherFile),
    // BitmapRangeComparison(BitmapRangeComparison),
    // StringTokenSearch(StringTokenSearch),
}

impl<'data> DecodableQuery<'data> {
    pub const fn new(data: &'data [u8]) -> Result<Self, u8> {
        let query_code = match QueryCode::from((data[0] >> 5) & 0x07) {
            Ok(code) => code,
            Err(x) => return Err(x),
        };
        Ok(unsafe {
            match query_code {
                QueryCode::ComparisonWithValue => DecodableQuery::ComparisonWithValue(
                    comparison_with_value::ComparisonWithValue::start_decoding_unchecked(data),
                ),
            }
        })
    }

    unsafe fn from_ptr(data: *const u8) -> Result<Self, u8> {
        let query_code = match QueryCode::from((*data.offset(0) >> 5) & 0x07) {
            Ok(code) => code,
            Err(x) => return Err(x),
        };
        Ok(match query_code {
            QueryCode::ComparisonWithValue => DecodableQuery::ComparisonWithValue(
                comparison_with_value::ComparisonWithValue::start_decoding_ptr(data),
            ),
        })
    }

    /// Decodes the size of the Item in bytes
    pub fn size(&self) -> usize {
        match self {
            Self::ComparisonWithValue(d) => d.size(),
        }
    }

    /// Fully decode the Item
    ///
    /// Returns the decoded data and the number of bytes consumed to produce it.
    pub fn complete_decoding(&self) -> (Query<'data>, usize) {
        match self {
            Self::ComparisonWithValue(d) => {
                let (op, size) = d.complete_decoding();
                (Query::ComparisonWithValue(op), size)
            }
        }
    }
}
