//! Contains a bit scatter/gather operation needed by the VDP that can be
//! replaced by a more performant implementation.

/// Simple Rust implementation.
pub fn simple_pattern_to_palette_indices(mut pattern: [u8; 4]) -> [u8; 8] {
    let mut result = [0u8; 8];
    for i in 0..8 {
        result[i] |= (pattern[0] & 0x80) >> 7;
        result[i] |= (pattern[1] & 0x80) >> 6;
        result[i] |= (pattern[2] & 0x80) >> 5;
        result[i] |= (pattern[3] & 0x80) >> 4;
        pattern[0] <<= 1;
        pattern[1] <<= 1;
        pattern[2] <<= 1;
        pattern[3] <<= 1;
    }
    result
}

/// This is provided as a static mutable variable so it can be replaced
/// with a higher performing implementation, such as the one in
/// the crate `attalus_x64`.
///
/// This is used by the VDP implementation to interpret bits in tiles and
/// sprites as indices into a palette.
///
/// It should do this bit scatter/gather operation.
/// b07,b06,b05,b04,b03,b02,b01,b00
/// b17,b16,b15,b14,b13,b12,b11,b10
/// b27,b26,b25,b24,b23,b22,b21,b20
/// b37,b36,b35,b34,b33,b32,b31,b30
/// ----->
///   0,  0,  0,  0,b37,b27,b17,b07
///   0,  0,  0,  0,b36,b26,b16,b06
///   0,  0,  0,  0,b35,b25,b15,b05
///   0,  0,  0,  0,b34,b24,b14,b04
///   0,  0,  0,  0,b33,b23,b13,b03
///   0,  0,  0,  0,b32,b22,b12,b02
///   0,  0,  0,  0,b31,b21,b11,b01
///   0,  0,  0,  0,b30,b20,b10,b00
/// Above, the bits are listed in logical order as they appear in the
/// arrays. If the arrays are reinterpreted as unsigned little endian
/// integers, the operation looks like this:
/// b37,b36,b35,b34,b33,b32,b31,b30
/// b27,b26,b25,b24,b23,b22,b21,b20
/// b17,b16,b15,b14,b13,b12,b11,b10
/// b07,b06,b05,b04,b03,b02,b01,b00
/// ----->
///   0,  0,  0,  0,b30,b20,b10,b00
///   0,  0,  0,  0,b31,b21,b11,b01
///   0,  0,  0,  0,b32,b22,b12,b02
///   0,  0,  0,  0,b33,b23,b13,b03
///   0,  0,  0,  0,b34,b24,b14,b04
///   0,  0,  0,  0,b35,b25,b15,b05
///   0,  0,  0,  0,b36,b26,b16,b06
///   0,  0,  0,  0,b37,b27,b17,b07
pub static mut PATTERN_TO_PALETTE_INDICES: fn(pattern: [u8; 4]) -> [u8; 8] =
    simple_pattern_to_palette_indices;
