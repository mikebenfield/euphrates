use super::*;

pub struct CodemastersMapper;

impl<M> SmsMapper<M> for CodemastersMapper
where
    M: SmsMemory + ?Sized,
{
    #[inline]
    fn write_reg(memory: &mut M, address: u16, value: u8) {
        use self::MemoryPage::*;
        let rom_pages = memory.rom_len() / 0x4000;
        let page = value & rom_pages as u8;
        match address {
            0x0000 => memory.map_page(0, Rom(page)),
            0x4000 => memory.map_page(1, Rom(page)),
            0x8000 => memory.map_page(2, Rom(page)),
            _ => {}
        }
    }

    fn default_mappings(memory: &mut M) {
        use self::MemoryPage::*;
        if memory.rom_len() >= 0x8000 {
            // at least 2 pages
            memory.map_page(0, Rom(0));
            memory.map_page(1, Rom(1));
            memory.map_page(0, Rom(0));
            memory.map_page(3, SystemRam);
        } else {
            memory.map_page(0, Rom(0));
            memory.map_page(0, Rom(0));
            memory.map_page(0, Rom(0));
            memory.map_page(3, SystemRam);
        }
    }
}

pub type CodemastersMemory16Impler<Memory> = SmsMapMemory16Impler<Memory, CodemastersMapper>;

pub type PointerCodemastersMemory16Impler = CodemastersMemory16Impler<PointerSmsMemory>;

pub type SimpleCodemastersMemory16Impler = CodemastersMemory16Impler<SmsMemoryState>;
