//! Memory maps for the Sega Master System.

use std;
use std::cell::UnsafeCell;
use std::sync::Arc;

use super::memory16::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum SmsMemoryMapper {
    Sega,
    Codemasters,
    Sg1000(usize),
}

impl Default for SmsMemoryMapper {
    #[inline(always)]
    fn default() -> Self {
        SmsMemoryMapper::Sega
    }
}

/// A 16 KiB page of memory.
///
/// This is used to indicate, for each of four 16 KiB slots of logical memory,
/// what physical page of memory it's mapped to.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum MemoryPage {
    /// The system RAM, mirrored across 16 KiB.
    SystemRam,

    /// The first 16 KiB page of RAM on the cartridge.
    ///
    /// Games with the Sega Memory Mapper have 0, 16, or 32 KiB of RAM on board.
    FirstCartridgeRam(u8),

    /// The second 16 KiB page of RAM on the cartridge.
    SecondCartridgeRam(u8),

    /// The first half of the slot is mapped to a page of ROM indicated by the
    /// parameter; the second half is mapped to 8 KiB of on-cartridge RAM.
    ///
    /// It seems some Codemasters games have 8 KiB of on-cartridge RAM.
    HalfCartridgeRam(u8),

    /// The page of ROM indicated by the parameter.
    Rom(u8),

    /// The page of ROM indicated by the parameter, except the first KiB of
    /// logical memory is mapped to the first KiB of physical ROM.
    RomButFirstKiB(u8),
}

mod _impl0 {
    use std::fmt::{Display, Error, Formatter};

    use super::MemoryPage;

    impl Default for MemoryPage {
        fn default() -> Self {
            MemoryPage::Rom(0)
        }
    }

    impl Display for MemoryPage {
        fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
            use self::MemoryPage::*;
            match self {
                SystemRam => "SystemRam".fmt(f),
                FirstCartridgeRam(x) => format_args!("FirstCartridgeRam({:>02X})", x).fmt(f),
                SecondCartridgeRam(x) => format_args!("SecondCartridgeRam({:>02X})", x).fmt(f),
                HalfCartridgeRam(x) => format_args!("HalfCartridgeRam({:>02X})", x).fmt(f),
                Rom(x) => format_args!("Rom({:>02X})", x).fmt(f),
                RomButFirstKiB(x) => format_args!("RomButFirstKiB({:>02X})", x).fmt(f),
            }
        }
    }

}

/// On-cartridge RAM, which can be dynamically allocated if needed.
///
/// This is for cartridges using the Sega Memory Mapper, which have 0, 16, or 32
/// KiB of RAM.
#[derive(Clone)]
pub enum MainCartridgeRam {
    Zero,
    One(Box<[u8; 0x4000]>),
    Two(Box<[u8; 0x4000]>, Box<[u8; 0x4000]>),
}

mod _impl1 {
    impl Default for super::MainCartridgeRam {
        fn default() -> Self {
            super::MainCartridgeRam::Zero
        }
    }

    #[derive(Hash, PartialEq, Serialize, Deserialize)]
    enum MainCartridgeRamDerive {
        Zero,
        One(Box<[[[u8; 0x20]; 0x20]; 0x10]>),
        Two(
            Box<[[[u8; 0x20]; 0x20]; 0x10]>,
            Box<[[[u8; 0x20]; 0x20]; 0x10]>,
        ),
    }

    impl_serde_via!{super::MainCartridgeRam, MainCartridgeRamDerive}
    impl_hash_via!{super::MainCartridgeRam, MainCartridgeRamDerive}
    impl_partial_eq_via!{super::MainCartridgeRam, MainCartridgeRamDerive}
    impl Eq for super::MainCartridgeRam {}
}

fn reg_sega<T>(memory: &mut T, address: u16, value: u8)
where
    T: SmsMemory + ?Sized,
{
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

fn reg_codemasters<T>(memory: &mut T, address: u16, value: u8)
where
    T: SmsMemory + ?Sized,
{
    use self::MemoryPage::*;
    let rom_pages = memory.rom_len() / 0x4000;
    let page = value % rom_pages as u8;
    match address {
        0x0000 => memory.map_page(0, Rom(page)),
        0x4000 => memory.map_page(1, Rom(page)),
        0x8000 => memory.map_page(2, Rom(page)),
        _ => {}
    }
}

pub fn default_mappings<M>(memory: &mut M)
where
    M: SmsMemory,
{
    use self::MemoryPage::*;
    match memory.mapper() {
        SmsMemoryMapper::Sega => {
            memory.set_system_ram_kib(8);
            memory.map_page(0, Rom(0));
            memory.map_page(1, Rom(1));
            memory.map_page(2, Rom(2));
            memory.map_page(3, SystemRam);
        }
        SmsMemoryMapper::Codemasters => {
            memory.set_system_ram_kib(8);
            memory.map_page(0, Rom(0));
            memory.map_page(1, Rom(1));
            memory.map_page(2, Rom(0));
            memory.map_page(3, SystemRam);
        }
        SmsMemoryMapper::Sg1000(x) => {
            use self::MemoryPage::*;
            use std::cmp::max;
            let kib = max(1, x);
            memory.set_system_ram_kib(kib);
            memory.map_page(0, Rom(0));
            memory.map_page(1, Rom(1));
            memory.map_page(2, Rom(2));
            memory.map_page(3, SystemRam);
        }
    }
}

/// The memory inside a Sega Master System.
///
/// Includes the cartridge ROM, 8 KiB of system RAM, 0, 16, or 32 KiB of "main"
/// cartridge RAM, and 0 or 8 KiB of "half" cartridge RAM.
pub trait SmsMemory: Memory16 {
    /// Read a byte of ROM.
    ///
    /// Panics if `index` is greater than the length of the ROM.
    fn rom_read(&self, index: usize) -> u8;

    /// Write a byte of ROM.
    ///
    /// Of course, this is not possible in actual hardware.
    ///
    /// Panics if `index` is greater than the length of the ROM.
    ///
    /// It is desirable that implementations share the same ROM behind an `Arc`
    /// when cloned. If that's the case, this method must
    /// XXX
    fn rom_write(&mut self, index: usize, value: u8);

    /// How much ROM on the cartridge, in bytes?
    fn rom_len(&self) -> usize;

    /// How much RAM is on the cartridge, in bytes?
    ///
    /// Note that this refers only to the 0, 16, or 32 KiB of RAM
    /// used in the Sega Memory Mapper. Since this may be dynamically
    /// allocated when requested by the ROM, it may increase.
    fn main_cartridge_ram_len(&self) -> usize;

    /// Read a byte of cartridge RAM.
    ///
    /// Panics if `index` is greater than the length of the RAM.
    fn main_cartridge_ram_read(&self, index: usize) -> u8;

    /// Write a byte of cartridge RAM.
    ///
    /// Panics if `index` is greater than the length of the RAM.
    fn main_cartridge_ram_write(&mut self, index: usize, value: u8);

    /// How much RAM is on the cartridge, in bytes?
    ///
    /// Note that this refers only to the 0, or 8 KiB of RAM used in the
    /// Codemasters Memory Mapper. Since this may be dynamically allocated when
    /// requested by the ROM, it may increase.
    fn half_cartridge_ram_len(&self) -> usize;

    /// Read a byte of cartridge RAM.
    ///
    /// Panics if `index` is greater than the length of the RAM.
    fn half_cartridge_ram_read(&self, index: usize) -> u8;

    /// Write a byte of cartridge RAM.
    ///
    /// Panics if `index` is greater than the length of the RAM.
    fn half_cartridge_ram_write(&mut self, index: usize, value: u8);

    /// How much system RAM, in bytes?
    ///
    /// (This is always a multiple of 0x400.)
    fn system_ram_len(&self) -> usize;

    /// Read a byte of system RAM.
    ///
    /// Panics if `index` is greater than the length of the RAM.
    fn system_ram_read(&self, index: usize) -> u8;

    /// Write a byte of system RAM.
    ///
    /// Panics if `index` is greater than the length of the RAM.
    fn system_ram_write(&mut self, index: usize, value: u8);

    fn state(&self) -> SmsMemoryState;

    /// Set how many KiB of RAM the system has.
    fn set_system_ram_kib(&mut self, kib: usize);

    fn mapper(&self) -> SmsMemoryMapper;

    fn set_mapper(&mut self, mapper: SmsMemoryMapper);

    /// What memory page is `slot` mapped to?
    ///
    /// Panics if `slot > 3`.
    fn page(&self, slot: u8) -> MemoryPage;

    /// Map `slot` to `page`.
    ///
    /// The default implementation, which should not be overridden, calls
    /// `map_page_impl`. In the case that the page indicated is from ROM, it
    /// takes the rom page indicated modulo the total number of ROM pages, and
    /// sends that to `map_page_impl`, which what an implementation of this
    /// trait should implement.
    fn map_page(&mut self, slot: u8, page: MemoryPage) {
        use self::MemoryPage::*;
        let rom_pages = (self.rom_len() / 0x4000) as u8;
        self.map_page_impl(
            slot,
            match page {
                Rom(x) => Rom(x % rom_pages),
                RomButFirstKiB(x) => RomButFirstKiB(x % rom_pages),
                x => x,
            },
        );
    }

    /// Map `slot` to `page` (for implementors of this trait; consumers should call
    /// `map_page`.
    ///
    /// Should be memory safe but panic if `page` refers to a ROM page that
    /// doesn't exist.
    fn map_page_impl(&mut self, slot: u8, page: MemoryPage);
}

/// Captures the state of the memory in the Master System.
///
/// In particular, it captures the ROM, system RAM, the main cartridge RAM, the
/// half page cartridge RAM, and the mappings of the four slots of logical
/// memory.
///
/// Implements `Memory16` and `SmsMemory`. Note that the implementation of
/// `Memory16` is done via a `match` statement to dispatch to the correct page
/// of memory, which is not the fastest approach.
///
/// This is suitable for serializing or for initializing the state of another
/// implementation via `SmsMemoryLoad`.
#[derive(Clone)]
pub struct SmsMemoryState {
    pub rom: Arc<Box<[u8]>>,
    pub system_ram: Box<[u8]>,
    pub main_cartridge_ram: MainCartridgeRam,
    pub half_cartridge_ram: Option<Box<[u8; 0x2000]>>,
    pub pages: [MemoryPage; 4],
    pub mapper: SmsMemoryMapper,
}

mod _impl2 {
    use super::*;
    use std::sync::Arc;

    #[derive(Hash, PartialEq, Serialize, Deserialize)]
    struct SmsMemoryStateDerive {
        pub rom: Arc<Box<[u8]>>,
        pub system_ram: Box<[u8]>,
        pub main_cartridge_ram: super::MainCartridgeRam,
        pub half_cartridge_ram: Option<Box<[[[u8; 0x20]; 0x10]; 0x10]>>,
        pub pages: [super::MemoryPage; 4],
        pub mapper: SmsMemoryMapper,
    }

    impl_serde_via!{super::SmsMemoryState, SmsMemoryStateDerive}
    impl_hash_via!{super::SmsMemoryState, SmsMemoryStateDerive}
    impl_partial_eq_via!{super::SmsMemoryState, SmsMemoryStateDerive}
    impl Eq for super::SmsMemoryState {}
}

impl SmsMemoryState {
    /// Are the mapped ROM pages in this `SmsMemoryState` valid?
    ///
    /// That is, are they smaller than the total number of pages in the ROM?
    pub fn check_valid(&self) -> Option<SmsMemoryLoadError> {
        use self::SmsMemoryLoadError::*;
        let rom_len = self.rom.len();
        let rom_pages = rom_len / 0x4000;
        if rom_len == 0 || rom_len > 0x400000 {
            return Some(InvalidRomSize(rom_len));
        }

        for (slot, page) in self.pages.iter().enumerate() {
            match page {
                MemoryPage::Rom(p) | MemoryPage::RomButFirstKiB(p) if *p > rom_pages as u8 => {
                    return Some(InvalidRomPageSelected {
                        slot: slot as u8,
                        selected: *p,
                        found: rom_pages as u8,
                    })
                }
                _ => {}
            }
        }

        None
    }

    fn ensure_half_page(&mut self) {
        if let None = self.half_cartridge_ram {
            self.half_cartridge_ram = Some(Box::new([0u8; 0x2000]));
        }
    }

    fn ensure_one_page(&mut self) {
        use self::MainCartridgeRam::*;
        if let Zero = self.main_cartridge_ram {
            self.main_cartridge_ram = One(Box::new([0u8; 0x4000]));
        }
    }

    fn ensure_two_pages(&mut self) {
        use self::MainCartridgeRam::*;
        use std::mem::swap;
        match &self.main_cartridge_ram {
            Zero => {
                self.main_cartridge_ram = Two(Box::new([0u8; 0x4000]), Box::new([0u8; 0x4000]));
            }
            One(_) => {
                let mut fake_ram = Zero;
                swap(&mut fake_ram, &mut self.main_cartridge_ram);
                let first_page = match fake_ram {
                    One(x) => x,
                    _ => unreachable!(),
                };
                self.main_cartridge_ram = Two(first_page, Box::new([0u8; 0x4000]));
            }
            _ => {}
        }
    }
}

impl SmsMemoryLoad for SmsMemoryState {
    #[inline(always)]
    fn load(state: SmsMemoryState) -> Result<Self, SmsMemoryLoadError> {
        if let Some(e) = state.check_valid() {
            Err(e)
        } else {
            Ok(state)
        }
    }
}

impl Memory16 for SmsMemoryState {
    fn read(&mut self, logical_address: u16) -> u8 {
        use self::MemoryPage::*;
        let slot = logical_address >> 14;
        let address = logical_address as usize & 0x3FFF;
        match self.pages[slot as usize] {
            SystemRam => {
                let len = self.system_ram_len();
                self.system_ram_read(address % len)
            }
            FirstCartridgeRam(_) => {
                self.ensure_one_page();
                self.main_cartridge_ram_read(address)
            }
            SecondCartridgeRam(_) => {
                self.ensure_two_pages();
                self.main_cartridge_ram_read(address + 0x4000)
            }
            HalfCartridgeRam(x) => {
                if address < 0x2000 {
                    self.rom_read(address + x as usize * 0x4000)
                } else {
                    self.ensure_half_page();
                    self.half_cartridge_ram_read(address - 0x2000)
                }
            }
            Rom(x) => self.rom_read(address + x as usize * 0x4000),
            RomButFirstKiB(x) => {
                if address < 0x400 {
                    self.rom_read(address)
                } else {
                    self.rom_read(address + x as usize * 0x4000)
                }
            }
        }
    }

    fn write(&mut self, logical_address: u16, value: u8) {
        use self::MemoryPage::*;
        memory_register_check(self, logical_address, value);
        let slot = logical_address >> 14;
        let address = logical_address as usize & 0x3FFF;
        match self.pages[slot as usize] {
            SystemRam => {
                let len = self.system_ram_len();
                self.system_ram_write(address % len, value)
            }
            FirstCartridgeRam(_) => {
                self.ensure_one_page();
                self.main_cartridge_ram_write(address, value)
            }
            SecondCartridgeRam(_) => {
                self.ensure_two_pages();
                self.main_cartridge_ram_write(address + 0x4000, value)
            }
            HalfCartridgeRam(_) => {
                if address >= 0x2000 {
                    self.ensure_half_page();
                    self.half_cartridge_ram_write(address - 0x2000, value)
                }
            }
            _ => {}
        }
    }
}

/// Check if writing to a register and update slots/pages as necessary.
#[inline]
pub fn memory_register_check<M>(memory: &mut M, logical_address: u16, value: u8)
where
    M: SmsMemory + ?Sized,
{
    match memory.mapper() {
        SmsMemoryMapper::Sega => reg_sega(memory, logical_address, value),
        SmsMemoryMapper::Codemasters => reg_codemasters(memory, logical_address, value),
        SmsMemoryMapper::Sg1000(_) => {}
    }
}

impl SmsMemory for SmsMemoryState {
    fn set_system_ram_kib(&mut self, kib: usize) {
        let len = kib * 0x400;
        self.system_ram = vec![0u8; len].into_boxed_slice();
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
        self.rom.len()
    }

    #[inline(always)]
    fn map_page_impl(&mut self, slot: u8, page: MemoryPage) {
        self.pages[slot as usize] = page;
    }
    #[inline(always)]
    fn rom_read(&self, index: usize) -> u8 {
        self.rom[index]
    }

    #[inline(always)]
    fn rom_write(&mut self, index: usize, value: u8) {
        Arc::make_mut(&mut self.rom)[index] = value;
    }

    #[inline(always)]
    fn main_cartridge_ram_len(&self) -> usize {
        use self::MainCartridgeRam::*;
        match self.main_cartridge_ram {
            Zero => 0,
            One(_) => 0x4000,
            Two(_, _) => 0x8000,
        }
    }

    fn main_cartridge_ram_read(&self, index: usize) -> u8 {
        use self::MainCartridgeRam::*;
        match &self.main_cartridge_ram {
            &Zero => panic!("index out of bounds: got {} but len 0", index),
            &One(ref x) => x[index],
            &Two(ref x, ref y) => if index < 0x4000 {
                x[index]
            } else if index < 0x8000 {
                y[index - 0x4000]
            } else {
                panic!("index out of bounds: got {} but len 0x8000", index)
            },
        }
    }

    fn main_cartridge_ram_write(&mut self, index: usize, value: u8) {
        use self::MainCartridgeRam::*;
        match &mut self.main_cartridge_ram {
            &mut Zero => panic!("index out of bounds: got {} but len 0", index),
            &mut One(ref mut x) => x[index] = value,
            &mut Two(ref mut x, ref mut y) => if index < 0x4000 {
                x[index] = value
            } else if index < 0x8000 {
                y[index - 0x4000] = value
            } else {
                panic!("index out of bounds: got {} but len 0x8000", index)
            },
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
            &Some(ref x) => x[index],
        }
    }

    #[inline(always)]
    fn half_cartridge_ram_write(&mut self, index: usize, value: u8) {
        match &mut self.half_cartridge_ram {
            &mut None => panic!("index out of bounds: got {} but len 0", index),
            &mut Some(ref mut x) => x[index] = value,
        }
    }

    #[inline(always)]
    fn system_ram_len(&self) -> usize {
        self.system_ram.len() * 0x400
    }

    #[inline(always)]
    fn system_ram_read(&self, index: usize) -> u8 {
        self.system_ram[index]
    }

    #[inline(always)]
    fn system_ram_write(&mut self, index: usize, value: u8) {
        self.system_ram[index] = value
    }

    #[inline(always)]
    fn state(&self) -> SmsMemoryState {
        self.clone()
    }
}

// This superfluous module with the `allow` attribute is necessary until the
// `fail` crate begins using `dyn trait` syntax
#[allow(bare_trait_objects)]
mod sms_memory_load_error {
    use super::*;

    /// Error generated by `SmsMemoryLoad`.
    #[derive(Debug, Fail)]
    pub enum SmsMemoryLoadError {
        /// The ROM size is not valid.
        ///
        /// The possible problems are:
        /// * It's 0.
        /// * It's not a multiple of 0x4000 (the size of a memory page).
        /// * It's bigger than 0x400000 (there are 8 bits to select a page, and each
        ///   16 bit logical address has 14 bits to select an offset within the
        ///   slot, leaving an effective 22 bit address).
        #[fail(
            display = "Invalid ROM size 0x{:x} (should be a positive multiple of 0x4000, no bigger than 0x400000)",
            _0
        )]
        InvalidRomSize(usize),

        #[fail(
            display = "Slot {} selected ROM page {}, but found only {} pages", slot, selected, found
        )]
        InvalidRomPageSelected { slot: u8, selected: u8, found: u8 },

        #[fail(display = "IO error while reading ROM file {}: {}", filename, io_error)]
        Io {
            filename: String,
            #[cause]
            io_error: std::io::Error,
        },
    }
}

pub use self::sms_memory_load_error::SmsMemoryLoadError;

/// A `SmsMemory` that can be initialized via a `SmsMemoryState` or a ROM.
pub trait SmsMemoryLoad: Sized {
    /// Load from an `SmsMemoryState`.
    ///
    /// This is provided in addition to `load_ref` since it may be possible for
    /// an implementation to use components from the `SmsMemoryState` and save
    /// the cost of allocating and copying.
    ///
    /// Note that there are default impementations of `load` and `load_ref`,
    /// but they call each other, so an implementer of this trait must
    /// override one of them.
    #[inline(always)]
    fn load(state: SmsMemoryState) -> Result<Self, SmsMemoryLoadError> {
        Self::load_ref(&state)
    }

    /// Load from a reference to a `SmsMemoryState`.
    ///
    /// Note that there are default impementations of `load` and `load_ref`,
    /// but they call each other, so an implementer of this trait must
    /// override one of them.
    #[inline(always)]
    fn load_ref(state: &SmsMemoryState) -> Result<Self, SmsMemoryLoadError> {
        Self::load(state.clone())
    }

    fn from_rom(rom: Box<[u8; 0x4000]>) -> Result<Self, SmsMemoryLoadError> {
        let state = SmsMemoryState {
            rom: Arc::new(rom),
            system_ram: Default::default(),
            main_cartridge_ram: Default::default(),
            half_cartridge_ram: Default::default(),
            pages: Default::default(),
            mapper: Default::default(),
        };
        return Self::load(state);
    }
}

/// An implementation of `SmsMemory` using pointer manipulation to map logical
/// memory addresses to physical memory addresses.
///
/// This is likely faster than the manual dispatch of `SmsMemoryState`, but I
/// haven't done a comparison yet.
pub struct PointerSmsMemory {
    state: UnsafeCell<SmsMemoryState>,
    scrap: Arc<[u8; 0x400]>,
    minislots: [[*const u8; 16]; 4],
    write_minislots: [[*mut u8; 16]; 4],
}

impl PointerSmsMemory {
    #[inline(always)]
    fn state(&self) -> &SmsMemoryState {
        use std::mem::transmute;
        unsafe { transmute(self.state.get()) }
    }

    #[inline(always)]
    fn state_mut(&mut self) -> &mut SmsMemoryState {
        use std::mem::transmute;
        unsafe { transmute(self.state.get()) }
    }
}

mod _impl3 {
    use std::cell::UnsafeCell;
    use std::hash::{Hash, Hasher};

    use super::*;

    unsafe impl Send for PointerSmsMemory {}
    unsafe impl Sync for PointerSmsMemory {}

    impl PartialEq for PointerSmsMemory {
        #[inline(always)]
        fn eq(&self, rhs: &Self) -> bool {
            self.state() == rhs.state()
        }
    }

    impl Eq for PointerSmsMemory {}

    impl Hash for PointerSmsMemory {
        #[inline(always)]
        fn hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.state().hash(state);
        }
    }

    impl Clone for PointerSmsMemory {
        #[inline]
        fn clone(&self) -> Self {
            use std::ptr::null;
            use std::ptr::null_mut;
            let mut other = PointerSmsMemory {
                state: UnsafeCell::new(self.state().clone()),
                scrap: self.scrap.clone(),
                minislots: [[null(); 16]; 4],
                write_minislots: [[null_mut(); 16]; 4],
            };
            other.reset_pointers();
            other
        }
    }

    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl Serialize for PointerSmsMemory {
        #[inline(always)]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.state().serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for PointerSmsMemory {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let state = <super::SmsMemoryState as Deserialize<'de>>::deserialize(deserializer)?;
            if let Some(e) = state.check_valid() {
                return Err(D::Error::custom(e));
            }

            use std::ptr::null;
            use std::ptr::null_mut;
            let mut memory = PointerSmsMemory {
                state: UnsafeCell::new(state),
                scrap: Arc::new([0u8; 0x400]),
                minislots: [[null(); 16]; 4],
                write_minislots: [[null_mut(); 16]; 4],
            };
            memory.reset_pointers();
            Ok(memory)
        }
    }

    impl From<SmsMemoryState> for PointerSmsMemory {
        fn from(state: SmsMemoryState) -> Self {
            use std::ptr::null;
            use std::ptr::null_mut;
            let mut smm = PointerSmsMemory {
                state: UnsafeCell::new(state),
                scrap: Arc::new([0u8; 0x400]),
                minislots: [[null(); 16]; 4],
                write_minislots: [[null_mut(); 16]; 4],
            };
            smm.reset_pointers();
            smm
        }
    }

    impl SmsMemoryLoad for PointerSmsMemory {
        fn load(state: SmsMemoryState) -> Result<Self, SmsMemoryLoadError> {
            use std::ptr::null;
            use std::ptr::null_mut;

            if let Some(e) = state.check_valid() {
                return Err(e);
            }

            let mut smm = PointerSmsMemory {
                state: UnsafeCell::new(state),
                scrap: Arc::new([0u8; 0x400]),
                minislots: [[null(); 16]; 4],
                write_minislots: [[null_mut(); 16]; 4],
            };
            smm.reset_pointers();
            Ok(smm)
        }
    }
}

impl PointerSmsMemory {
    fn force_map_page(&mut self, slot: u8, page: MemoryPage) {
        use self::MemoryPage::*;
        use std::mem::transmute;
        use std::ops::Deref;

        let minislots = &mut self.minislots[slot as usize];
        let write_minislots = &mut self.write_minislots[slot as usize];
        let state: &mut SmsMemoryState = unsafe { &mut *self.state.get() };

        // transmuting from an immutable pointer to a mutable one is highly
        // illegal, but since I'm never going to read `scrap` it surely doesn't
        // matter
        let scrap_ptr: *mut u8 = unsafe { transmute(self.scrap.deref()) };

        match page {
            SystemRam => {
                let kib = state.system_ram.len();
                for i in 0..16 {
                    let offset = (i % kib) * 0x400;
                    let ptr: *mut u8 =
                        unsafe { state.system_ram.as_mut_ptr().offset(offset as isize) };
                    minislots[i] = ptr;
                    write_minislots[i] = ptr;
                }
            }
            FirstCartridgeRam(_) => {
                state.ensure_one_page();
                let ptr: *mut u8 = match &mut state.main_cartridge_ram {
                    &mut MainCartridgeRam::One(ref mut x) => &mut x[0],
                    &mut MainCartridgeRam::Two(ref mut x, _) => &mut x[0],
                    _ => unreachable!(),
                };
                for i in 0..16 {
                    let p = unsafe { ptr.offset(i as isize * 0x400) };
                    minislots[i] = p;
                    write_minislots[i] = p;
                }
            }
            SecondCartridgeRam(_) => {
                state.ensure_two_pages();
                let ptr: *mut u8 = match &mut state.main_cartridge_ram {
                    &mut MainCartridgeRam::Two(_, ref mut x) => &mut x[0],
                    _ => unreachable!(),
                };
                for i in 0..16 {
                    let p = unsafe { ptr.offset(i as isize * 0x400) };
                    minislots[i] = p;
                    write_minislots[i] = p;
                }
            }
            HalfCartridgeRam(page) => {
                state.ensure_half_page();
                let ptr0: *const u8 = unsafe { state.rom.as_ptr().offset(page as isize * 0x4000) };
                let ptr1: *mut u8 = match &mut state.half_cartridge_ram {
                    &mut Some(ref mut x) => &mut x[0],
                    _ => unreachable!(),
                };
                for i in 0..8 {
                    let p0 = unsafe { ptr0.offset(i as isize * 0x400 + 0x2000) };
                    minislots[i] = p0;
                    write_minislots[i] = scrap_ptr;
                    let p1 = unsafe { ptr1.offset(i as isize * 0x400) };
                    minislots[i + 8] = p1;
                    write_minislots[i + 8] = p1;
                }
            }
            Rom(page) => {
                let ptr: *const u8 = unsafe { state.rom.as_ptr().offset(page as isize * 0x4000) };
                for i in 0..16 {
                    let p = unsafe { ptr.offset(i as isize * 0x400) };
                    minislots[i] = p;
                    write_minislots[i] = scrap_ptr;
                }
            }
            RomButFirstKiB(page) => {
                minislots[0] = state.rom.as_ptr();
                write_minislots[0] = scrap_ptr;
                let ptr: *const u8 = unsafe { state.rom.as_ptr().offset(page as isize * 0x4000) };
                for i in 1..16 {
                    let p = unsafe { ptr.offset(i as isize * 0x400) };
                    minislots[i] = p;
                    write_minislots[i] = scrap_ptr;
                }
            }
        }
        state.pages[slot as usize] = page;
    }

    #[inline(always)]
    fn reset_pointers(&mut self) {
        for i in 0..4 {
            let page = self.page(i);
            self.force_map_page(i, page);
        }
    }
}

impl Memory16 for PointerSmsMemory {
    #[inline]
    fn read(&mut self, logical_address: u16) -> u8 {
        use std::mem::transmute;
        unsafe {
            let minislots: &[*const u8; 64] = transmute(&self.minislots);
            *minislots[logical_address as usize >> 10].offset(logical_address as isize & 0x3FF)
        }
    }

    #[inline]
    fn write(&mut self, logical_address: u16, value: u8) {
        use std::mem::transmute;
        memory_register_check(self, logical_address, value);
        unsafe {
            let write_minislots: &mut [*mut u8; 64] = transmute(&mut self.write_minislots);
            *write_minislots[logical_address as usize >> 10]
                .offset(logical_address as isize & 0x3FF) = value
        }
    }
}

impl SmsMemory for PointerSmsMemory {
    fn set_system_ram_kib(&mut self, kib: usize) {
        self.state_mut().set_system_ram_kib(kib);
        self.reset_pointers();
    }

    #[inline(always)]
    fn mapper(&self) -> SmsMemoryMapper {
        self.state().mapper()
    }

    #[inline(always)]
    fn set_mapper(&mut self, mapper: SmsMemoryMapper) {
        self.state_mut().set_mapper(mapper)
    }

    #[inline]
    fn page(&self, slot: u8) -> MemoryPage {
        self.state().page(slot)
    }

    #[inline]
    fn rom_len(&self) -> usize {
        self.state().rom_len()
    }

    #[inline]
    fn map_page_impl(&mut self, slot: u8, page: MemoryPage) {
        // let's not go through a bunch of moving around pointers unless we
        // really have to
        if self.page(slot) != page {
            self.force_map_page(slot, page)
        }
    }
    #[inline]
    fn rom_read(&self, index: usize) -> u8 {
        self.state().rom_read(index)
    }

    #[inline]
    fn rom_write(&mut self, index: usize, value: u8) {
        // can't forward to `state` because we need to check whether we should
        // reset our pointers
        {
            let state = self.state_mut();
            if let Some(rom) = Arc::get_mut(&mut state.rom) {
                // we have the only copy of the ROM; just mutate
                rom[index] = value;
                return;
            }
            // there may be other copies; we must clone and reset our pointers
            Arc::make_mut(&mut state.rom)[index] = value;
        }
        self.reset_pointers();
    }

    #[inline]
    fn main_cartridge_ram_len(&self) -> usize {
        self.state().main_cartridge_ram_len()
    }

    #[inline]
    fn main_cartridge_ram_read(&self, index: usize) -> u8 {
        self.state().main_cartridge_ram_read(index)
    }

    #[inline]
    fn main_cartridge_ram_write(&mut self, index: usize, value: u8) {
        self.state_mut().main_cartridge_ram_write(index, value)
    }

    #[inline]
    fn half_cartridge_ram_len(&self) -> usize {
        self.state().half_cartridge_ram_len()
    }

    #[inline]
    fn half_cartridge_ram_read(&self, index: usize) -> u8 {
        self.state().half_cartridge_ram_read(index)
    }

    #[inline]
    fn half_cartridge_ram_write(&mut self, index: usize, value: u8) {
        self.state_mut().half_cartridge_ram_write(index, value)
    }

    #[inline]
    fn system_ram_len(&self) -> usize {
        self.state().system_ram_len()
    }

    #[inline]
    fn system_ram_read(&self, index: usize) -> u8 {
        self.state().system_ram_read(index)
    }

    #[inline]
    fn system_ram_write(&mut self, index: usize, value: u8) {
        self.state_mut().system_ram_write(index, value)
    }

    #[inline]
    fn state(&self) -> SmsMemoryState {
        self.state().state()
    }
}
