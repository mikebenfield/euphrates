use super::*;

/// The Sega memory mapper.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SegaMapper;

impl<M> SmsMapper<M> for SegaMapper
where
    M: SmsMemory + ?Sized,
{
    #[inline]
    fn write_reg(memory: &mut M, address: u16, value: u8) {
        use self::MemoryPage::*;
        let rom_pages = memory.rom_len() / 0x4000;
        let page = value % rom_pages as u8;
        match address {
            0xFFFD => memory.map_page(0, RomButFirstKiB(page)),
            0xFFFE => memory.map_page(1, Rom(page)),
            0xFFFF => match memory.page(2) {
                FirstCartridgeRam(_) => memory.map_page(2, FirstCartridgeRam(page)),
                SecondCartridgeRam(_) => memory.map_page(2, SecondCartridgeRam(page)),
                _ => memory.map_page(2, Rom(page)),
            },
            0xFFFC => {
                let ram_slot2 = value & 0x8 != 0;
                let cartridge_ram_slot3 = value & 0x10 != 0;
                let second_ram_page_slot2 = value & 0x4 != 0;
                let slot2_rom_page = match memory.page(2) {
                    Rom(x) => x,
                    FirstCartridgeRam(x) => x,
                    SecondCartridgeRam(x) => x,
                    _ => 0,
                };
                match (ram_slot2, second_ram_page_slot2) {
                    (false, _) => memory.map_page(2, Rom(slot2_rom_page)),
                    (true, false) => memory.map_page(2, FirstCartridgeRam(slot2_rom_page)),
                    (true, true) => memory.map_page(2, SecondCartridgeRam(slot2_rom_page)),
                }
                if cartridge_ram_slot3 {
                    memory.map_page(3, FirstCartridgeRam(0));
                } else {
                    memory.map_page(3, SystemRam);
                }
            }
            _ => {}
        }
    }

    fn default_mappings(memory: &mut M) {
        use self::MemoryPage::*;
        if memory.rom_len() >= 0xC000 {
            // at least 3 pages
            memory.map_page(0, Rom(0));
            memory.map_page(1, Rom(1));
            memory.map_page(2, Rom(2));
            memory.map_page(3, SystemRam);
        } else {
            memory.map_page(0, Rom(0));
            memory.map_page(0, Rom(0));
            memory.map_page(0, Rom(0));
            memory.map_page(3, SystemRam);
        }
    }
}

pub type SegaMemory16Impler<Memory> = SmsMapMemory16Impler<Memory, SegaMapper>;

pub type PointerSegaMemory16Impler = SegaMemory16Impler<PointerSmsMemory>;

pub type SimpleSegaMemory16Impler = SegaMemory16Impler<SmsMemoryState>;

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    struct Mem<T>(T);

    impl<U> Memory16Impl for Mem<U>
    where
        U: SmsMemory + Memory16 + 'static,
    {
        type Impler = SegaMemory16Impler<U>;

        fn close<F, T>(&self, f: F) -> T
        where
            F: FnOnce(&Self::Impler) -> T,
        {
            SegaMemory16Impler::iclose(&self.0, |z| f(z))
        }

        fn close_mut<F, T>(&mut self, f: F) -> T
        where
            F: FnOnce(&mut Self::Impler) -> T,
        {
            SegaMemory16Impler::iclose_mut(&mut self.0, |z| f(z))
        }
    }

    impl<U> SmsMemoryImpl for Mem<U>
    where
        U: SmsMemory + 'static,
    {
        type Impler = U;

        fn close<F, T>(&self, f: F) -> T
        where
            F: FnOnce(&Self::Impler) -> T,
        {
            f(&self.0)
        }

        fn close_mut<F, T>(&mut self, f: F) -> T
        where
            F: FnOnce(&mut Self::Impler) -> T,
        {
            f(&mut self.0)
        }
    }

    fn build_mmap<T>() -> Mem<T>
    where
        T: SmsMemory + Memory16 + From<SmsMemoryState> + 'static,
    {
        let mut rom = [[0u8; 0x4000]; 4]; // 64 KiB (8 8KiB impl-pages or 4 16KiB sega-pages)
        {
            use std::mem::transmute;
            let rom_view: &mut [u8; 0x10000] = unsafe { transmute(&mut rom) };
            rom_view[0x2000] = 1;
            rom_view[0x4000] = 2;
            rom_view[0x6000] = 3;
            rom_view[0x8000] = 4;
            rom_view[0xA000] = 5;
            rom_view[0xC000] = 6;
            rom_view[0xE000] = 7;

            rom_view[0x9E02] = 100;
        }

        let state = SmsMemoryState::from_rom(Box::new(rom)).unwrap();
        let mut smm = Mem(T::from(state));
        SegaMapper::default_mappings(&mut smm);
        smm
    }

    #[test]
    fn read() {
        read_t::<SmsMemoryState>();
        read_t::<PointerSmsMemory>();
    }

    fn read_t<T>()
    where
        T: SmsMemory + Memory16 + From<SmsMemoryState> + 'static,
    {
        let smm = &mut build_mmap::<T>();

        // read impl-slot 0
        assert!(smm.read(0) == 0);

        // read impl-slot 1
        assert!(smm.read(0x2000) == 1);

        // read impl-slot 2
        assert!(smm.read(0x4000) == 2);

        // read impl-slot 3
        assert!(smm.read(0x6000) == 3);

        // read impl-slot 4
        assert!(smm.read(0x8000) == 4);

        // read impl-slot 5
        assert!(smm.read(0xA000) == 5);

        // read impl-slot 6 (should be system memory)
        assert!(smm.read(0xC000) == 0);

        // read impl-slot 7 (should be system memory)
        assert!(smm.read(0xE000) == 0);
    }

    #[test]
    fn reg_ffff() {
        reg_ffff_t::<SmsMemoryState>();
        reg_ffff_t::<PointerSmsMemory>();
    }

    fn reg_ffff_t<T>()
    where
        T: SmsMemory + Memory16 + From<SmsMemoryState> + 'static,
    {
        let smm = &mut build_mmap::<T>();
        smm.write(0xFFFF, 3); // sega-slot 2 should now map to sega-page 3
        assert!(smm.read(0x8000) == 6);
        assert!(smm.read(0xA000) == 7);
        smm.write(0xFFFF, 0); // sega-slot 2 should now map to sega-page 0
        assert!(smm.read(0x8000) == 0);
        assert!(smm.read(0xA000) == 1);
        smm.write(0xFFFF, 2); // sega-slot 2 should now map to sega-page 2
        assert!(smm.read(0x9E02) == 100);
        assert!(smm.read(0x8000) == 4);
        assert!(smm.read(0xA000) == 5);
    }

    #[test]
    fn reg_fffe() {
        reg_fffe_t::<SmsMemoryState>();
        reg_fffe_t::<PointerSmsMemory>();
    }

    fn reg_fffe_t<T>()
    where
        T: SmsMemory + Memory16 + From<SmsMemoryState> + 'static,
    {
        let smm = &mut build_mmap::<T>();
        smm.write(0xFFFE, 3); // sega-slot 1 should now map to sega-page 3
        assert!(smm.read(0x4000) == 6);
        assert!(smm.read(0x6000) == 7);
        smm.write(0xFFFE, 0); // sega-slot 1 should now map to sega-page 0
        assert!(smm.read(0x4000) == 0);
        assert!(smm.read(0x6000) == 1);
    }

    #[test]
    fn reg_fffd() {
        reg_fffd_t::<SmsMemoryState>();
        reg_fffd_t::<PointerSmsMemory>();
    }

    fn reg_fffd_t<T>()
    where
        T: SmsMemory + Memory16 + From<SmsMemoryState> + 'static,
    {
        let smm = &mut build_mmap::<T>();
        smm.write(0xFFFD, 1); // sega-slot 0 should now map to sega-page 1
        assert!(smm.read(0x0000) == 0); // except the first KiB
        assert!(smm.read(0x2000) == 3);
        smm.write(0xFFFD, 0); // sega-slot 0 should now map to sega-page 0
        assert!(smm.read(0x0000) == 0);
        assert!(smm.read(0x2000) == 1);
    }

    #[test]
    fn reg_fffc() {
        reg_fffc_t::<SmsMemoryState>();
        reg_fffc_t::<PointerSmsMemory>();
    }

    fn reg_fffc_t<T>()
    where
        T: SmsMemory + Memory16 + From<SmsMemoryState> + 'static,
    {
        let smm = &mut build_mmap::<T>();
        smm.write(0xFFFC, 0b1000); // sega-slot 2 mapped to sega-page 0 of cartridge RAM
        assert!(smm.read(0x8000) == 0);
        smm.write(0x8000, 102);
        assert!(smm.read(0x8000) == 102);

        smm.write(0xFFFC, 0); // sega-slot 2 mapped back to sega-page 2 of ROM
        assert!(smm.read(0x8000) == 4);
        smm.write(0, 17);
        assert!(smm.read(0x8000) == 4); // which should not be writable

        smm.write(0xFFFC, 0b1000); // back to sega-page 0 of cartridge RAM
        assert!(smm.read(0x8000) == 102);

        smm.write(0xFFFC, 0b1100); // to sega-page 1 of cartridge RAM
        assert!(smm.read(0x8000) == 0);
        smm.write(0x8000, 103);
        assert!(smm.read(0x8000) == 103);

        smm.write(0xFFFC, 0b1000); // back to sega-page 0 of cartridge RAM
        assert!(smm.read(0x8000) == 102);
    }
}
