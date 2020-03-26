#[cfg(test)]
use crate::codec::{Codec, ParseValue};

#[cfg(test)]
pub fn test_item<T: Codec + std::fmt::Debug + std::cmp::PartialEq>(item: T, data: &[u8]) {
    assert_eq!(item.encode_to_box()[..], *data);
    assert_eq!(
        T::decode(&data).expect("should be parsed without error"),
        ParseValue {
            value: item,
            size: data.len(),
        }
    );
}
