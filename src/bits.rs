pub fn to16(lo: u8, hi: u8) -> u16 {
    ((hi as u16) << 8) | (lo as u16)
}

pub fn to8(x: u16) -> (u8, u8) {
    ((x & 0xFF) as u8, ((x & 0xFF00) >> 8) as u8)
}

pub fn set_bit(dest: &mut u8, bit: u8) {
    *dest |= 1 << bit;
}

pub fn clear_bit(dest: &mut u8, bit: u8) {
    *dest &= !(1 << bit);
}

pub fn assign_bit(dest: &mut u8, bit1: u8, source: u8, bit2: u8) {
    let bitflag = source >> bit2 & 1;
    let bitflag_positioned = bitflag << bit1;
    *dest = (*dest & !(1 << bit1)) | bitflag_positioned;
}
