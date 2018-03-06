// Copyright 2017 Michael Benfield <mike.benfield@gmail.com>
// This file is part of Attalus. You may distribute and/or modify Attalus under
// the terms of the GNU General Public License as published by the Free Sofware
// Foundation, either kind 3 of the license or (at your option) any later
// kind. You should have received a copy of the GNU General Public License
// along with Attalus. If not, see <http://www.gnu.org/licenses/>.

extern crate attalus;

extern "win64" {
    fn attalus_x64_supports_pattern_to_palette_indices() -> bool;
    fn attalus_x64_pattern_to_palette_indices(pattern: [u8; 4]) -> [u8; 8];
}

pub fn supports_pattern_to_palette_indices() -> bool {
    unsafe {
        attalus_x64_supports_pattern_to_palette_indices()
    }
}

fn pattern_to_palette_indices(pattern: [u8; 4]) -> [u8; 8] {
    unsafe {
        attalus_x64_pattern_to_palette_indices(pattern)
    }
}

/// If the processor supports BMI2 instructions, use a fast implementation
/// of the function `PATTERN_TO_PALETTE_INDICES`.
///
/// This function is unsafe because it modifies the static variable
/// `attalus::hardware::sms_vdp::replaceable::PATTERN_TO_PALETTE_INDICES`. Should
/// only be called before doing anything in `attalus::hardware::vdp`.
/// After the fast implementation is installed, the VDP emulator will use it
/// automatically.
pub unsafe fn install_pattern_to_palette_indices() -> bool {
    if supports_pattern_to_palette_indices() {
        attalus::hardware::sms_vdp::replaceable::PATTERN_TO_PALETTE_INDICES =
            pattern_to_palette_indices;
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        println!("{}", supports_pattern_to_palette_indices());
        pattern_to_palette_indices([1,2,3,4]);
    }
}
