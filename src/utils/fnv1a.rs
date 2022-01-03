pub const FNV_PRIME: u32 = 16777619;
pub const OFFSET_BASIS: u32 = 2166136261;

/// Performs a 32-bit FNV-1a hash, as done by the original game.
pub fn fnv1a_hash(buffer: &[u8]) -> u32 {
    let mut result = OFFSET_BASIS;
    for &byte in buffer {
        // NOTE: BF2 additionally ORs every byte with 0x20, presumably to make
        //       the encoding case-insensitive, but it does actually screw up
        //       characters like underscores, which don't fall for such nasty
        //       ASCII tricks. That's why you use tolower(int) in C++!
        result ^= (byte | 0x20) as u32;
        result = result.wrapping_mul(FNV_PRIME);
    }
    result
}
