// Standard action flags
pub const GROUP: u8 = 0x80;
pub const RESPONSE: u8 = 0x40;

// Tag action flags
pub const END_OF_PACKET: u8 = 0x80;
pub const ERROR: u8 = 0x40;

// Chunk action flags
pub const CHUNK_CONTINUE: u8 = 0x00;
pub const CHUNK_START: u8 = 0x40;
pub const CHUNK_END: u8 = 0x80;
pub const CHUNK_START_END: u8 = CHUNK_START + CHUNK_END;

// Logic action flags
pub const LOGIC_OR: u8 = 0x00;
pub const LOGIC_XOR: u8 = 0x40;
pub const LOGIC_NOR: u8 = 0x80;
pub const LOGIC_NAND: u8 = 0xC0;

// Indirect forward
pub const OVERLOAD: u8 = 0x80;

// Status flags
pub const STATUS_ACTION: u8 = 0x00;
pub const STATUS_INTERFACE: u8 = 0x40;

// Queries
pub const QUERY_MASK: u8 = 1 << 4;
pub const QUERY_SIGNED_DATA: u8 = 1 << 3;
pub const QUERY_COMPARISON_TYPE: u8 = 0x07;
