pub const FNV_PRIME: u32 = 16777619;
pub const OFFSET_BASIS: u32 = 2166136261;

/// Performs a 32-bit FNV-1a hash, as done by the original game.
///
/// ## Example
/// ```
/// use zenit_utils::fnv1a_hash;
///
/// let speeder = "all_fly_snowspeeder";
/// let hash = fnv1a_hash(speeder.as_bytes());
/// assert_eq!(hash, 0x266561d8);
/// ```
pub fn fnv1a_hash(buffer: &[u8]) -> u32 {
    let mut result = OFFSET_BASIS;
    for &byte in buffer {
        // NOTE: BF2 additionally ORs every byte with 0x20, presumably to make
        //       the encoding case-insensitive, but it does actually screw up
        //       characters like underscores, which don't fall for such nasty
        //       ASCII tricks. That's why you use tolower(int) in C++!
        //
        //       To be fair, that probably was expected behavior. I still don't
        //       like it lol
        result ^= (byte | 0x20) as u32;
        result = result.wrapping_mul(FNV_PRIME);
    }
    result
}

pub trait FnvHashExt {
    /// Verifies if given string's FNV-1a hash matches this value.
    ///
    /// ## Example:
    /// ```
    /// use zenit_utils::FnvHashExt;
    ///
    /// let hash: u32 = 0x266561d8;
    /// assert!(hash.fnv1a_matches("all_fly_snowspeeder"))
    /// ```
    fn fnv1a_matches(self, string: &str) -> bool;
}

impl FnvHashExt for u32 {
    fn fnv1a_matches(self, string: &str) -> bool {
        fnv1a_hash(string.as_bytes()) == self
    }
}
