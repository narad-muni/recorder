pub fn u32_to_bytes(ms: u32) -> [u8; 4] {
    ms.to_be_bytes()
}

pub fn bytes_to_u32(bytes: [u8; 4]) -> u32 {
    u32::from_be_bytes(bytes)
}
