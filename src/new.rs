#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// Size does not fit in a varint
    SizeTooBig,
    /// Offset does not fit in a varint
    OffsetTooBig,
    /// Mask size is different from item size attribute
    MaskBadSize,
    /// Size of the data does not fit in a varint
    DataTooBig,
    StartGreaterThanStop,
    /// Bitmap size is different from what is expected by given the start and stop parameters
    BitmapBadSize,
    /// An NLS state is required by the specified Addressee. Please provide one.
    MissingNlsState,
    /// The dash7 id type specified is too big
    IdTypeTooBig,
}
