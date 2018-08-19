use std::ops::Deref;
use std::ptr;
use std::sync::Arc;

use euphrates::hardware::memory16::Memory16;
use euphrates::hardware::sms_memory::{
    self, MemoryPage, SmsMemory, SmsMemoryLoad, SmsMemoryLoadError, SmsMemoryMapper, SmsMemoryState,
};

use super::*;
use traits::{UnsafeMemory, VirtualMemory};

type Physical<T> = <Main as VirtualMemory<T>>::PhysicalMemory;
type Logical<T> = <Main as VirtualMemory<T>>::LogicalMemory;

const PAGE_SIZE: usize = <Main as VirtualMemory<u8>>::PAGE_SIZE;

#[allow(non_upper_case_globals)]
const physical: unsafe fn(pages: usize, write: bool, execute: bool) -> Physical<u8> =
    <Main as VirtualMemory<u8>>::allocate_physical;

#[allow(non_upper_case_globals)]
const logical: unsafe fn(pages: usize, write: bool, execute: bool) -> Logical<u8> =
    <Main as VirtualMemory<u8>>::allocate_logical;

#[inline(always)]
unsafe fn dup<T: Copy>(slice: &[T], execute: bool) -> Physical<T> {
    <Main as VirtualMemory<T>>::dup(slice, execute)
}

#[inline(always)]
fn create_slice<T: Copy>(phys: &Physical<T>) -> Box<[T]> {
    <Main as VirtualMemory<T>>::create_slice(phys)
}

#[allow(non_upper_case_globals)]
const map: unsafe fn(
    logical: &mut Logical<u8>,
    logical_page_start: usize,
    physical: &Physical<u8>,
    physical_page_start: usize,
    len: usize,
    write: bool,
    execute: bool,
) = <Main as VirtualMemory<u8>>::map;

pub struct Rom(Physical<u8>);

impl Clone for Rom {
    fn clone(&self) -> Self {
        let mut other = unsafe { physical(self.0.len(), true, false) };
        unsafe {
            ptr::copy_nonoverlapping(self.0.as_ptr(), other.as_mut_ptr(), self.0.len());
        }
        Rom(other)
    }
}

pub enum MainCartridgeRam {
    Zero,
    One(Physical<u8>),
    Two(Physical<u8>, Physical<u8>),
}

pub struct SmsVirtualMemory {
    rom: Arc<Rom>,
    system_ram: Physical<u8>,
    main_cartridge_ram: MainCartridgeRam,
    half_cartridge_ram: Option<Physical<u8>>,

    /// Half a Sega page of memory, to be written to but never read
    scrap: Arc<Physical<u8>>,

    /// One sega page, the first KiB of which is copied from the first KiB of
    /// rom and the rest of which is copied from some other page of ROM
    rom_but_1: Option<Physical<u8>>,

    pages: [MemoryPage; 4],
    mapper: SmsMemoryMapper,
    write_mem: Logical<u8>,
    read_mem: Logical<u8>,
}

impl SmsVirtualMemory {
    fn ensure_half_page(&mut self) {
        if let None = self.half_cartridge_ram {
            unsafe {
                self.half_cartridge_ram = Some(physical(0x2000, true, false));
            }
        }
    }

    fn ensure_one_page(&mut self) {
        use self::MainCartridgeRam::*;
        if let Zero = self.main_cartridge_ram {
            unsafe {
                self.main_cartridge_ram = One(physical(0x4000, true, false));
            }
        }
    }

    fn ensure_two_pages(&mut self) {
        use self::MainCartridgeRam::*;
        use std::mem::swap;
        match &self.main_cartridge_ram {
            Zero => unsafe {
                self.main_cartridge_ram =
                    Two(physical(0x4000, true, false), physical(0x4000, true, false));
            },
            One(_) => {
                let mut fake_ram = Zero;
                swap(&mut fake_ram, &mut self.main_cartridge_ram);
                let first_page = match fake_ram {
                    One(x) => x,
                    _ => unreachable!(),
                };
                unsafe {
                    self.main_cartridge_ram = Two(first_page, physical(0x4000, true, false));
                }
            }
            _ => {}
        }
    }

    #[inline]
    fn remap(&mut self) {
        for i in 0..4 {
            let page = self.page(i);
            self.force_map_page(i, page);
        }
    }

    fn force_map_page(&mut self, slot: u8, page: MemoryPage) {
        use self::MemoryPage::*;

        let slot = slot as usize;

        let logical_offset = slot as usize * 0x4000;

        fn map_both(
            read_mem: &mut Logical<u8>,
            write_mem: &mut Logical<u8>,
            physical_mem: &Physical<u8>,
            logical_offset: usize,
            len: usize,
        ) {
            unsafe {
                map(read_mem, logical_offset, physical_mem, 0, len, false, false);
                map(write_mem, logical_offset, physical_mem, 0, len, true, false);
            }
        };
        match page {
            SystemRam => {
                for i in 0..0x4000 / self.system_ram.len() {
                    map_both(
                        &mut self.read_mem,
                        &mut self.write_mem,
                        &self.system_ram,
                        logical_offset + i * self.system_ram.len(),
                        self.system_ram.len(),
                    )
                }
            }
            FirstCartridgeRam(_) => {
                self.ensure_one_page();
                let phys = match &self.main_cartridge_ram {
                    &MainCartridgeRam::One(ref x) => x,
                    &MainCartridgeRam::Two(ref x, _) => x,
                    _ => unreachable!(),
                };
                map_both(
                    &mut self.read_mem,
                    &mut self.write_mem,
                    phys,
                    logical_offset,
                    0x4000,
                );
            }
            SecondCartridgeRam(_) => {
                self.ensure_two_pages();
                let phys = match &self.main_cartridge_ram {
                    &MainCartridgeRam::Two(_, ref x) => x,
                    _ => unreachable!(),
                };
                map_both(
                    &mut self.read_mem,
                    &mut self.write_mem,
                    phys,
                    logical_offset,
                    0x4000,
                );
            }
            HalfCartridgeRam(page) => {
                self.ensure_half_page();
                let rom_offset = page as usize * 0x4000;
                // rom in the first half sega page
                unsafe {
                    map(
                        &mut self.write_mem,
                        logical_offset,
                        self.scrap.deref(),
                        0,
                        0x2000,
                        true,
                        false,
                    );
                    map(
                        &mut self.read_mem,
                        logical_offset,
                        &self.rom.deref().0,
                        rom_offset,
                        0x2000,
                        false,
                        true,
                    );
                }

                // ram in the second half sega page
                match &mut self.half_cartridge_ram {
                    Some(ref mut x) => {
                        map_both(
                            &mut self.read_mem,
                            &mut self.write_mem,
                            x,
                            logical_offset + 0x2000,
                            0x2000,
                        );
                    }
                    _ => unreachable!(),
                }
            }
            Rom(page) => {
                let native_rom_offset = page as usize * 0x4000;
                unsafe {
                    map(
                        &mut self.write_mem,
                        logical_offset,
                        self.scrap.deref(),
                        0,
                        0x2000,
                        true,
                        false,
                    );
                    map(
                        &mut self.write_mem,
                        logical_offset + 0x2000,
                        self.scrap.deref(),
                        0,
                        0x2000,
                        true,
                        false,
                    );
                    map(
                        &mut self.read_mem,
                        logical_offset,
                        &self.rom.deref().0,
                        native_rom_offset,
                        0x4000,
                        false,
                        false,
                    );
                }
            }
            RomButFirstKiB(page) => {
                let rom_offset = page as usize * 0x4000 + 0x400;
                let mut phys = unsafe { physical(0x4000, true, false) };
                unsafe {
                    // one KiB from the beginning of ROM
                    ptr::copy_nonoverlapping(self.rom.deref().0.as_ptr(), phys.as_mut_ptr(), 0x400);
                    // the rest from the indicated page
                    ptr::copy_nonoverlapping(
                        self.rom.deref().0.as_ptr().offset(rom_offset as isize),
                        phys.as_mut_ptr().offset(0x400),
                        0x3C00,
                    );
                    map(
                        &mut self.write_mem,
                        logical_offset,
                        self.scrap.deref(),
                        0,
                        0x2000,
                        true,
                        false,
                    );
                    map(
                        &mut self.write_mem,
                        logical_offset + 0x2000,
                        self.scrap.deref(),
                        0,
                        0x2000,
                        true,
                        false,
                    );
                    map(
                        &mut self.read_mem,
                        logical_offset,
                        &mut phys,
                        0,
                        0x4000,
                        false,
                        false,
                    );
                }
                self.rom_but_1 = Some(phys);
            }
        }
        self.pages[slot] = page;
    }
}

impl Memory16 for SmsVirtualMemory {
    #[inline]
    fn read(&mut self, logical_address: u16) -> u8 {
        unsafe { self.read_mem.get_unchecked(logical_address as usize) }
    }

    #[inline]
    fn write(&mut self, logical_address: u16, value: u8) {
        sms_memory::memory_register_check(self, logical_address, value);
        unsafe {
            self.write_mem
                .set_unchecked(logical_address as usize, value);
        }
    }
}

impl SmsMemory for SmsVirtualMemory {
    fn set_system_ram_kib(&mut self, kib: usize) {
        use std::cmp::min;
        let size = 0x400 * kib;
        let len = min(size, PAGE_SIZE);
        unsafe {
            self.system_ram = physical(len, true, false);
        }
        self.remap();
    }

    #[inline(always)]
    fn mapper(&self) -> SmsMemoryMapper {
        self.mapper
    }

    #[inline(always)]
    fn set_mapper(&mut self, mapper: SmsMemoryMapper) {
        self.mapper = mapper;
    }

    #[inline(always)]
    fn page(&self, slot: u8) -> MemoryPage {
        self.pages[slot as usize]
    }

    #[inline(always)]
    fn rom_len(&self) -> usize {
        self.rom.deref().0.len()
    }

    #[inline]
    fn map_page_impl(&mut self, slot: u8, page: MemoryPage) {
        if self.page(slot) != page {
            self.force_map_page(slot, page)
        }
    }

    #[inline]
    fn rom_read(&self, index: usize) -> u8 {
        unsafe { self.rom.deref().0.get(index) }
    }

    #[inline(always)]
    fn rom_write(&mut self, index: usize, value: u8) {
        unsafe {
            Arc::make_mut(&mut self.rom).0.set(index, value);
        }
    }

    #[inline]
    fn main_cartridge_ram_len(&self) -> usize {
        use self::MainCartridgeRam::*;
        match self.main_cartridge_ram {
            Zero => 0,
            One(_) => 0x4000,
            Two(_, _) => 0x8000,
        }
    }

    #[inline]
    fn main_cartridge_ram_read(&self, index: usize) -> u8 {
        use self::MainCartridgeRam::*;
        unsafe {
            match &self.main_cartridge_ram {
                &Zero => panic!("index out of bounds: got {} but len 0", index),
                &One(ref x) => x.get(index),
                &Two(ref x, ref y) => if index < 0x4000 {
                    x.get(index)
                } else if index < 0x8000 {
                    y.get(index - 0x4000)
                } else {
                    panic!("index out of bounds: got {} but len 0x8000", index)
                },
            }
        }
    }

    #[inline]
    fn main_cartridge_ram_write(&mut self, index: usize, value: u8) {
        use self::MainCartridgeRam::*;
        unsafe {
            match &mut self.main_cartridge_ram {
                &mut Zero => panic!("index out of bounds: got {} but len 0", index),
                &mut One(ref mut x) => x.set(index, value),
                &mut Two(ref mut x, ref mut y) => if index < 0x4000 {
                    x.set(index, value)
                } else if index < 0x8000 {
                    y.set(index - 0x4000, value)
                } else {
                    panic!("index out of bounds: got {} but len 0x8000", index)
                },
            }
        }
    }

    #[inline(always)]
    fn half_cartridge_ram_len(&self) -> usize {
        match self.half_cartridge_ram {
            None => 0,
            Some(_) => 0x2000,
        }
    }

    #[inline(always)]
    fn half_cartridge_ram_read(&self, index: usize) -> u8 {
        match &self.half_cartridge_ram {
            &None => panic!("index out of bounds: got {} but len 0", index),
            &Some(ref x) => unsafe { x.get(index) },
        }
    }

    #[inline(always)]
    fn half_cartridge_ram_write(&mut self, index: usize, value: u8) {
        match &mut self.half_cartridge_ram {
            &mut None => panic!("index out of bounds: got {} but len 0", index),
            &mut Some(ref mut x) => unsafe { x.set(index, value) },
        }
    }

    #[inline(always)]
    fn system_ram_len(&self) -> usize {
        self.system_ram.len()
    }

    #[inline(always)]
    fn system_ram_read(&self, index: usize) -> u8 {
        unsafe { self.system_ram.get(index) }
    }

    #[inline(always)]
    fn system_ram_write(&mut self, index: usize, value: u8) {
        unsafe { self.system_ram.set(index, value) }
    }

    #[inline]
    fn state(&self) -> SmsMemoryState {
        let main_cartridge_ram = match self.main_cartridge_ram {
            MainCartridgeRam::Zero => sms_memory::MainCartridgeRam::Zero,
            MainCartridgeRam::One(ref x) => {
                let mut array = [0u8; 0x4000];
                unsafe {
                    ptr::copy_nonoverlapping(x.as_ptr(), array.as_mut_ptr(), 0x4000);
                }
                sms_memory::MainCartridgeRam::One(Box::new(array))
            }
            MainCartridgeRam::Two(ref x, ref y) => {
                let mut array0 = [0u8; 0x4000];
                unsafe {
                    ptr::copy_nonoverlapping(x.as_ptr(), array0.as_mut_ptr(), 0x4000);
                }
                let mut array1 = [0u8; 0x4000];
                unsafe {
                    ptr::copy_nonoverlapping(y.as_ptr(), array1.as_mut_ptr(), 0x4000);
                }
                sms_memory::MainCartridgeRam::Two(Box::new(array0), Box::new(array1))
            }
        };
        let half_cartridge_ram = match self.half_cartridge_ram {
            None => None,
            Some(ref x) => {
                let mut array = [0u8; 0x2000];
                unsafe {
                    ptr::copy_nonoverlapping(x.as_ptr(), array.as_mut_ptr(), 0x2000);
                }
                Some(Box::new(array))
            }
        };

        SmsMemoryState {
            rom: Arc::new(create_slice(&self.rom.0)),
            system_ram: create_slice(&self.system_ram),
            main_cartridge_ram,
            half_cartridge_ram,
            pages: self.pages.clone(),
            mapper: self.mapper,
        }
    }
}

impl SmsMemoryLoad for SmsVirtualMemory {
    #[inline(always)]
    fn load_ref(state: &SmsMemoryState) -> Result<Self, SmsMemoryLoadError> {
        if let Some(e) = state.check_valid() {
            return Err(e);
        }

        unsafe {
            let rom = dup::<u8>(state.rom.deref(), false);

            let system_ram = dup::<u8>(state.system_ram.deref(), false);

            let main_cartridge_ram = match state.main_cartridge_ram {
                sms_memory::MainCartridgeRam::Zero => MainCartridgeRam::Zero,
                sms_memory::MainCartridgeRam::One(ref x) => {
                    MainCartridgeRam::One(dup::<u8>(x.deref(), false))
                }
                sms_memory::MainCartridgeRam::Two(ref x, ref y) => {
                    MainCartridgeRam::Two(dup::<u8>(x.deref(), false), dup::<u8>(y.deref(), false))
                }
            };

            let half_cartridge_ram = match state.half_cartridge_ram {
                Some(ref x) => Some(dup::<u8>(x.deref(), false)),
                None => None,
            };

            let scrap = physical(0x2000, true, false);

            let mut vm = SmsVirtualMemory {
                rom: Arc::new(Rom(rom)),
                system_ram,
                main_cartridge_ram,
                half_cartridge_ram,
                scrap: Arc::new(scrap),
                rom_but_1: None,
                pages: state.pages,
                mapper: state.mapper,
                write_mem: logical(0x10000, true, false),
                read_mem: logical(0x10000, false, false),
            };

            vm.remap();
            Ok(vm)
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use traits::tests as t;

    // tests for the unsafe stuff
    #[test]
    fn alloc_read() {
        t::alloc_read::<Main>();
    }

    #[test]
    fn read_write() {
        t::read_write::<Main>();
    }

    #[test]
    fn shared() {
        t::shared::<Main>();
    }

    #[test]
    fn shared2() {
        t::shared2::<Main>();
    }

    #[test]
    fn shared_u32() {
        t::shared_u32::<Main>();
    }

    // tests for the external interface
    fn make_state() -> SmsMemoryState {
        let mut rom0 = [0u8; 0x4000];
        rom0[1] = 1;
        rom0[2] = 2;
        rom0[0x3FFF] = 3;
        let mut rom1 = [0u8; 0x4000];
        rom1[1] = 11;
        rom1[2] = 12;
        rom1[0x3FFF] = 13;

        let mut ram0 = [0u8; 0x400];
        ram0[1] = 21;
        ram0[2] = 22;

        let mut cart_ram = [0u8; 0x4000];
        cart_ram[1] = 31;
        cart_ram[2] = 32;

        SmsMemoryState {
            rom: Arc::new(Box::new([rom0, rom1])),
            system_ram: Box::new([
                ram0,
                ram0.clone(),
                ram0.clone(),
                ram0.clone(),
                ram0.clone(),
                ram0.clone(),
                ram0.clone(),
                ram0.clone(),
            ]),
            main_cartridge_ram: sms_memory::MainCartridgeRam::One(Box::new(cart_ram)),
            half_cartridge_ram: None,
            pages: Default::default(),
            mapper: Default::default(),
        }
    }

    #[test]
    fn state() {
        let s = make_state();
        let mut vm = SmsVirtualMemory::load(s).unwrap();
        assert_eq!(vm.read(0), 0);
        assert_eq!(vm.read(1), 1);
        assert_eq!(vm.read(2), 2);
        assert_eq!(vm.read(0x3FFF), 3);
        assert_eq!(vm.read(0x4000), 0);
        assert_eq!(vm.read(0x4001), 1);
        assert_eq!(vm.read(0x4002), 2);
        assert_eq!(vm.read(0x7FFF), 3);
        assert_eq!(vm.read(0x8000), 0);
        assert_eq!(vm.read(0x8001), 1);
        assert_eq!(vm.read(0x8002), 2);
        assert_eq!(vm.read(0xBFFF), 3);
        assert_eq!(vm.read(0xC000), 0);
        assert_eq!(vm.read(0xC001), 1);
        assert_eq!(vm.read(0xC002), 2);
        assert_eq!(vm.read(0xFFFF), 3);
        vm.map_page(3, MemoryPage::SystemRam);
        assert_eq!(vm.read(0xC000), 0);
        assert_eq!(vm.read(0xC001), 21);
        assert_eq!(vm.read(0xC002), 22);
        assert_eq!(vm.read(0xC400), 0);
        assert_eq!(vm.read(0xC401), 21);
        assert_eq!(vm.read(0xC402), 22);
        assert_eq!(vm.read(0xE000), 0);
        assert_eq!(vm.read(0xE001), 21);
        assert_eq!(vm.read(0xE002), 22);

        vm.write(0xC001, 51);
        vm.map_page(2, MemoryPage::SystemRam);
        assert_eq!(vm.read(0x8001), 51);

        vm.map_page(1, MemoryPage::FirstCartridgeRam(0));
        assert_eq!(vm.read(0x4000), 0);
        assert_eq!(vm.read(0x4001), 31);
        assert_eq!(vm.read(0x4002), 32);

        vm.map_page(0, MemoryPage::RomButFirstKiB(1));
        assert_eq!(vm.read(0), 0);
        assert_eq!(vm.read(1), 1);
        assert_eq!(vm.read(2), 2);
        assert_eq!(vm.read(0x3FFF), 13);
    }
}
