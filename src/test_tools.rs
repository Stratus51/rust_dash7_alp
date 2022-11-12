#[cfg(test)]
use crate::codec::{Codec, WithSize};

#[cfg(test)]
pub fn test_item<T: Codec + std::fmt::Debug + std::cmp::PartialEq>(item: T, data: &[u8])
where
    T::Error: std::fmt::Debug,
{
    assert_eq!(item.encode()[..], *data);
    assert_eq!(
        T::decode(&data).expect("should be parsed without error"),
        WithSize {
            value: item,
            size: data.len(),
        }
    );
}
